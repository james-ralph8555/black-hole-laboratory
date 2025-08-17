use std::collections::VecDeque;

#[cfg(target_arch = "wasm32")]
use web_sys::Performance;

// Platform-specific timing
#[cfg(not(target_arch = "wasm32"))]
type PlatformInstant = std::time::Instant;

#[cfg(target_arch = "wasm32")]
type PlatformInstant = f64; // JavaScript timestamp

const MAX_FRAME_SAMPLES: usize = 60;

#[derive(Clone)]
pub struct TimingSample {
    pub cpu_time_ms: f32,
    pub gpu_time_ms: Option<f32>,
    pub update_time_ms: f32,
    pub render_encode_time_ms: f32,
}

pub struct Profiler {
    // CPU timing
    frame_start: Option<PlatformInstant>,
    update_start: Option<PlatformInstant>,
    render_encode_start: Option<PlatformInstant>,

    #[cfg(target_arch = "wasm32")]
    performance: Performance,

    // GPU timing (WebGPU timestamp queries)
    timestamp_query_set: Option<wgpu::QuerySet>,
    timestamp_buffer: Option<wgpu::Buffer>,
    timestamp_staging_buffer: Option<wgpu::Buffer>,
    pending_gpu_queries: VecDeque<u32>, // Frame indices waiting for GPU results

    // Timing history
    timing_samples: VecDeque<TimingSample>,
    current_sample: TimingSample,

    // Statistics
    pub avg_cpu_time_ms: f32,
    pub avg_gpu_time_ms: f32,
    pub max_cpu_time_ms: f32,
    pub max_gpu_time_ms: f32,

    frame_count: u32,
    gpu_timing_supported: bool,
}

impl Profiler {
    pub fn new(device: &wgpu::Device) -> Self {
        #[cfg(target_arch = "wasm32")]
        let performance = web_sys::window().unwrap().performance().unwrap();
        let gpu_timing_supported = device.features().contains(wgpu::Features::TIMESTAMP_QUERY);

        let (timestamp_query_set, timestamp_buffer, timestamp_staging_buffer) =
            if gpu_timing_supported {
                // Create query set for GPU timestamps
                let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("Timestamp Query Set"),
                    ty: wgpu::QueryType::Timestamp,
                    count: 2, // Start and end timestamps
                });

                // Buffer to store query results on GPU
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Timestamp Buffer"),
                    size: 16, // 2 timestamps * 8 bytes each
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                // Staging buffer to read results back to CPU
                let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Timestamp Staging Buffer"),
                    size: 16,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                (Some(query_set), Some(buffer), Some(staging_buffer))
            } else {
                (None, None, None)
            };

        Self {
            frame_start: None,
            update_start: None,
            render_encode_start: None,

            #[cfg(target_arch = "wasm32")]
            performance,
            timestamp_query_set,
            timestamp_buffer,
            timestamp_staging_buffer,
            pending_gpu_queries: VecDeque::new(),
            timing_samples: VecDeque::new(),
            current_sample: TimingSample {
                cpu_time_ms: 0.0,
                gpu_time_ms: None,
                update_time_ms: 0.0,
                render_encode_time_ms: 0.0,
            },
            avg_cpu_time_ms: 0.0,
            avg_gpu_time_ms: 0.0,
            max_cpu_time_ms: 0.0,
            max_gpu_time_ms: 0.0,
            frame_count: 0,
            gpu_timing_supported,
        }
    }

    pub fn begin_frame(&mut self) {
        self.frame_start = Some(self.now());
        self.current_sample = TimingSample {
            cpu_time_ms: 0.0,
            gpu_time_ms: None,
            update_time_ms: 0.0,
            render_encode_time_ms: 0.0,
        };
    }

    fn now(&self) -> PlatformInstant {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::Instant::now()
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.performance.now()
        }
    }

    fn elapsed_ms(&self, start: PlatformInstant) -> f32 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            (start.elapsed().as_secs_f64() * 1000.0) as f32
        }
        #[cfg(target_arch = "wasm32")]
        {
            (self.now() - start) as f32
        }
    }

    pub fn begin_update(&mut self) {
        self.update_start = Some(self.now());
    }

    pub fn end_update(&mut self) {
        if let Some(start) = self.update_start.take() {
            self.current_sample.update_time_ms = self.elapsed_ms(start);
        }
    }

    pub fn begin_render_encode(&mut self) {
        self.render_encode_start = Some(self.now());
    }

    pub fn end_render_encode(&mut self) {
        if let Some(start) = self.render_encode_start.take() {
            self.current_sample.render_encode_time_ms = self.elapsed_ms(start);
        }
    }

    pub fn begin_gpu_timing(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(query_set) = &self.timestamp_query_set {
            encoder.write_timestamp(query_set, 0);
        }
    }

    pub fn end_gpu_timing(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(query_set) = &self.timestamp_query_set {
            encoder.write_timestamp(query_set, 1);
            self.pending_gpu_queries.push_back(self.frame_count);
        }
    }

    pub fn resolve_gpu_timing(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if let (Some(query_set), Some(buffer)) = (&self.timestamp_query_set, &self.timestamp_buffer)
        {
            encoder.resolve_query_set(query_set, 0..2, buffer, 0);
        }
    }

    pub fn end_frame(&mut self) {
        if let Some(start) = self.frame_start.take() {
            self.current_sample.cpu_time_ms = self.elapsed_ms(start);
        }

        self.frame_count += 1;

        // Add current sample to history
        self.timing_samples.push_back(self.current_sample.clone());
        if self.timing_samples.len() > MAX_FRAME_SAMPLES {
            self.timing_samples.pop_front();
        }

        // Update statistics
        self.update_statistics();
    }

    pub fn try_read_gpu_timing(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.gpu_timing_supported || self.pending_gpu_queries.is_empty() {
            return;
        }

        let Some(staging_buffer) = &self.timestamp_staging_buffer else {
            return;
        };
        let Some(buffer) = &self.timestamp_buffer else {
            return;
        };

        // Copy from GPU buffer to staging buffer for reading
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("GPU Timing Copy"),
        });
        encoder.copy_buffer_to_buffer(buffer, 0, staging_buffer, 0, 16);
        queue.submit(std::iter::once(encoder.finish()));

        // Map the staging buffer for reading (async operation)
        let buffer_slice = staging_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |result| {
            if result.is_err() {
                log::warn!("Failed to map GPU timing buffer");
            }
        });

        // Poll the device to process the map request
        device.poll(wgpu::Maintain::Wait);

        // Try to read the results
        let buffer_view = buffer_slice.get_mapped_range();
        let timestamps: &[u64] = bytemuck::cast_slice(&buffer_view);
        if timestamps.len() >= 2 && timestamps[0] > 0 && timestamps[1] > timestamps[0] {
            let start_ns = timestamps[0];
            let end_ns = timestamps[1];
            let gpu_time_ms = (end_ns - start_ns) as f32 / 1_000_000.0;

            // Apply to the most recent sample
            if let Some(latest_sample) = self.timing_samples.back_mut() {
                latest_sample.gpu_time_ms = Some(gpu_time_ms);
            }

            self.pending_gpu_queries.pop_front();
        }
        drop(buffer_view);
        staging_buffer.unmap();
    }

    fn update_statistics(&mut self) {
        if self.timing_samples.is_empty() {
            return;
        }

        let mut cpu_sum = 0.0;
        let mut gpu_sum = 0.0;
        let mut gpu_count = 0;
        let mut max_cpu: f32 = 0.0;
        let mut max_gpu: f32 = 0.0;

        for sample in &self.timing_samples {
            cpu_sum += sample.cpu_time_ms;
            max_cpu = max_cpu.max(sample.cpu_time_ms);

            if let Some(gpu_time) = sample.gpu_time_ms {
                gpu_sum += gpu_time;
                gpu_count += 1;
                max_gpu = max_gpu.max(gpu_time);
            }
        }

        self.avg_cpu_time_ms = cpu_sum / self.timing_samples.len() as f32;
        self.avg_gpu_time_ms = if gpu_count > 0 {
            gpu_sum / gpu_count as f32
        } else {
            0.0
        };
        self.max_cpu_time_ms = max_cpu;
        self.max_gpu_time_ms = max_gpu;
    }

    pub fn get_latest_sample(&self) -> Option<&TimingSample> {
        self.timing_samples.back()
    }

    pub fn is_gpu_timing_supported(&self) -> bool {
        self.gpu_timing_supported
    }

    pub fn get_frame_count(&self) -> u32 {
        self.frame_count
    }
}
