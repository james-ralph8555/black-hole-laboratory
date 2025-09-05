// Import the initializer and the function you want to run
import init, { run } from './pkg/index.js';

function checkWebGLSupport() {
  try {
    // Use a temporary canvas so we don't touch the main one
    const testCanvas = document.createElement('canvas');
    // Try WebGL2 first (required by wgpu's GL backend)
    const gl2 = testCanvas.getContext('webgl2', { alpha: false, antialias: true });
    if (gl2) {
      return { ok: true };
    }
    // Fall back to WebGL1 presence check to provide a clearer message
    const gl1 = testCanvas.getContext('webgl') || testCanvas.getContext('experimental-webgl');
    if (gl1) {
      return { ok: false, reason: 'WebGL 2 is not available (only WebGL 1 detected).' };
    }
    return { ok: false, reason: 'WebGL is not available in this browser.' };
  } catch (e) {
    return { ok: false, reason: `WebGL check failed: ${e}` };
  }
}

function showFatalError(message) {
  const loading = document.getElementById('loading-screen');
  if (!loading) return;
  const title = loading.querySelector('.loading-title');
  const subtitle = loading.querySelector('.loading-subtitle');
  const spinner = loading.querySelector('.loading-spinner');
  const text = loading.querySelector('.loading-text');

  if (spinner) spinner.style.display = 'none';
  if (title) title.textContent = 'GRAVITYLENS';
  if (subtitle) subtitle.textContent = 'Unable to start renderer';
  if (text) {
    text.style.animation = 'none';
    text.style.color = '#ff6b6b';
    text.textContent = message + ' Please enable hardware acceleration and WebGL, then reload. Chrome/Firefox are recommended.';
  }
}

function isControlFlowException(e) {
  const msg = (e && e.message) ? e.message : String(e || '');
  return msg.includes('Using exceptions for control flow');
}

function setupCanvas() {
  const canvas = document.getElementById('wasm-canvas');
  if (!canvas) return;
  
  function resizeCanvas() {
    const displayWidth = window.innerWidth;
    const displayHeight = window.innerHeight;
    
    // Set the CSS size to fill the viewport
    canvas.style.width = displayWidth + 'px';
    canvas.style.height = displayHeight + 'px';
    
    // Set canvas resolution to match viewport (with device pixel ratio consideration)
    const devicePixelRatio = window.devicePixelRatio || 1;
    const renderWidth = Math.floor(displayWidth * devicePixelRatio);
    const renderHeight = Math.floor(displayHeight * devicePixelRatio);
    
    canvas.width = renderWidth;
    canvas.height = renderHeight;
    
    console.log(`Canvas: CSS ${displayWidth}x${displayHeight}, Render ${renderWidth}x${renderHeight}, DPR ${devicePixelRatio}`);
  }
  
  // Initial resize
  resizeCanvas();
  
  // Listen for window resize events
  window.addEventListener('resize', resizeCanvas);
  
  // Also listen for orientation change on mobile
  window.addEventListener('orientationchange', () => {
    setTimeout(resizeCanvas, 100);
  });
}

// Help overlay management
let helpVisible = false; // Start with help hidden
let helpFlashVisible = true; // Start with flash message visible
let helpStartupTimer = 3000; // 3 seconds in milliseconds

function isMobileDevice() {
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) || 
         ('ontouchstart' in window) || 
         (navigator.maxTouchPoints > 0);
}

function setupHelpOverlay() {
  const helpOverlay = document.getElementById('help-overlay');
  const helpFlash = document.getElementById('help-flash');
  const touchControlsOverlay = document.getElementById('touch-controls-overlay');
  if (!helpOverlay || !helpFlash || !touchControlsOverlay) return;
  
  // Start with full help hidden and appropriate startup message visible
  helpOverlay.style.display = 'none';
  
  const isMobile = isMobileDevice();
  
  if (isMobile) {
    // On mobile, show touch control regions instead of text message
    helpFlash.style.display = 'none';
    touchControlsOverlay.style.display = 'block';
  } else {
    // On desktop, show text message
    helpFlash.style.display = 'block';
    touchControlsOverlay.style.display = 'none';
  }
  
  // Auto-fade startup message after timer
  setTimeout(() => {
    if (helpFlashVisible) {
      if (isMobile) {
        // Fade out touch controls overlay
        touchControlsOverlay.style.transition = 'opacity 1s ease-out';
        touchControlsOverlay.style.opacity = '0';
        setTimeout(() => {
          if (helpFlashVisible) {
            touchControlsOverlay.style.display = 'none';
            helpFlashVisible = false;
          }
        }, 1000);
      } else {
        // Fade out text message
        helpFlash.style.transition = 'opacity 1s ease-out';
        helpFlash.style.opacity = '0';
        setTimeout(() => {
          if (helpFlashVisible) {
            helpFlash.style.display = 'none';
            helpFlashVisible = false;
          }
        }, 1000);
      }
    }
  }, helpStartupTimer);
}

// Global functions for WASM to call
window.toggleHelp = function() {
  const helpOverlay = document.getElementById('help-overlay');
  const helpFlash = document.getElementById('help-flash');
  const touchControlsOverlay = document.getElementById('touch-controls-overlay');
  if (!helpOverlay || !helpFlash || !touchControlsOverlay) return;
  
  helpVisible = !helpVisible;
  helpOverlay.style.display = helpVisible ? 'block' : 'none';
  
  // Fade out startup message when user manually toggles help
  if (helpFlashVisible) {
    const isMobile = isMobileDevice();
    
    if (isMobile) {
      // Fade out touch controls overlay
      touchControlsOverlay.style.transition = 'opacity 0.3s ease-out';
      touchControlsOverlay.style.opacity = '0';
      setTimeout(() => {
        touchControlsOverlay.style.display = 'none';
      }, 300);
    } else {
      // Fade out text message
      helpFlash.style.transition = 'opacity 0.3s ease-out';
      helpFlash.style.opacity = '0';
      setTimeout(() => {
        helpFlash.style.display = 'none';
      }, 300);
    }
    
    helpFlashVisible = false;
  }
  
  // Disable auto-hide timer when manually toggled
  helpStartupTimer = 0;
};

window.setHelpVisible = function(visible) {
  const helpOverlay = document.getElementById('help-overlay');
  if (!helpOverlay) return;
  
  helpVisible = visible;
  helpOverlay.style.display = helpVisible ? 'block' : 'none';
  
  // Only hide startup message if help is being shown (user manually opened help)
  // Don't hide on automatic state sync when help is false
  if (visible && helpFlashVisible) {
    const helpFlash = document.getElementById('help-flash');
    const touchControlsOverlay = document.getElementById('touch-controls-overlay');
    const isMobile = isMobileDevice();
    
    if (isMobile && touchControlsOverlay) {
      // Fade out touch controls overlay
      touchControlsOverlay.style.transition = 'opacity 0.3s ease-out';
      touchControlsOverlay.style.opacity = '0';
      setTimeout(() => {
        touchControlsOverlay.style.display = 'none';
      }, 300);
    } else if (helpFlash) {
      // Fade out text message
      helpFlash.style.transition = 'opacity 0.3s ease-out';
      helpFlash.style.opacity = '0';
      setTimeout(() => {
        helpFlash.style.display = 'none';
      }, 300);
    }
    
    helpFlashVisible = false;
    
    // Disable the auto-hide timer since user manually opened help
    helpStartupTimer = 0;
  }
};

window.updateDebugInfo = function(position, orientation, lastKey, fps, renderWidth, renderHeight, velocity) {
  document.getElementById('debug-position').textContent = 
    `Position: (${position[0].toFixed(2)}, ${position[1].toFixed(2)}, ${position[2].toFixed(2)})`;
  document.getElementById('debug-orientation').textContent = 
    `Orientation: Yaw ${orientation[0].toFixed(1)}°, Pitch ${orientation[1].toFixed(1)}°`;
  document.getElementById('debug-velocity').textContent = 
    `Velocity: (${velocity[0].toFixed(2)}, ${velocity[1].toFixed(2)}, ${velocity[2].toFixed(2)})`;
  document.getElementById('debug-lastkey').textContent = 
    `Last Key: ${lastKey || 'None'}`;
  document.getElementById('debug-fps').textContent = 
    `FPS: ${fps.toFixed(1)}`;
  document.getElementById('debug-resolution').textContent = 
    `Resolution: ${renderWidth.toFixed(0)}x${renderHeight.toFixed(0)}`;
};

window.updateFpsCounter = function(fps, visible) {
  const fpsCounter = document.getElementById('fps-counter');
  if (fpsCounter) {
    fpsCounter.textContent = `FPS: ${fps.toFixed(1)}`;
    fpsCounter.style.display = visible ? 'block' : 'none';
  }
};

// Profiling averages
let profilingHistory = {
  cpu: [],
  update: [],
  render: [],
  maxSamples: 60
};

window.updateProfilingInfo = function(cpuTime, gpuTime, updateTime, renderTime, gpuSupported) {
  // Update running averages
  profilingHistory.cpu.push(cpuTime);
  profilingHistory.update.push(updateTime);
  profilingHistory.render.push(renderTime);
  
  // Keep only recent samples
  if (profilingHistory.cpu.length > profilingHistory.maxSamples) {
    profilingHistory.cpu.shift();
    profilingHistory.update.shift();
    profilingHistory.render.shift();
  }
  
  // Calculate averages
  const avgCpu = profilingHistory.cpu.reduce((a, b) => a + b, 0) / profilingHistory.cpu.length;
  const avgUpdate = profilingHistory.update.reduce((a, b) => a + b, 0) / profilingHistory.update.length;
  const avgRender = profilingHistory.render.reduce((a, b) => a + b, 0) / profilingHistory.render.length;
  
  document.getElementById('profiling-cpu').textContent = `CPU: ${cpuTime.toFixed(4)}ms (avg: ${avgCpu.toFixed(4)}ms)`;
  
  if (gpuSupported && gpuTime > 0) {
    document.getElementById('profiling-gpu').textContent = `GPU: ${gpuTime.toFixed(4)}ms`;
    document.getElementById('profiling-support').textContent = `GPU Timing: Supported`;
  } else {
    document.getElementById('profiling-gpu').textContent = `GPU: ${gpuSupported ? 'Pending...' : 'N/A'}`;
    document.getElementById('profiling-support').textContent = `GPU Timing: ${gpuSupported ? 'Supported' : 'Not Supported'}`;
  }
  
  document.getElementById('profiling-update').textContent = `Update: ${updateTime.toFixed(4)}ms (avg: ${avgUpdate.toFixed(4)}ms)`;
  document.getElementById('profiling-render').textContent = `Render: ${renderTime.toFixed(4)}ms (avg: ${avgRender.toFixed(4)}ms)`;
};

window.setProfilingVisible = function(visible) {
  const profilingOverlay = document.getElementById('profiling-overlay');
  if (profilingOverlay) {
    profilingOverlay.style.display = visible ? 'block' : 'none';
  }
};

// Debug control sliders - global reference to WASM module
let wasmModule = null;

function setupDebugControls() {
  // Helper function to blur slider after interaction to restore keyboard focus
  function setupSlider(sliderId, valueId, updateCallback, formatter) {
    const slider = document.getElementById(sliderId);
    const valueDisplay = document.getElementById(valueId);
    if (slider && valueDisplay) {
      slider.addEventListener('input', (e) => {
        const value = parseFloat(e.target.value);
        valueDisplay.textContent = formatter(value);
        if (wasmModule && updateCallback) {
          updateCallback(value);
        }
      });
      
      // Blur the slider when user stops interacting to restore keyboard focus to canvas
      slider.addEventListener('change', () => {
        slider.blur();
      });
      
      // Also blur on mouse leave to ensure keyboard focus returns
      slider.addEventListener('mouseleave', () => {
        slider.blur();
      });
    }
  }

  // FOV slider
  setupSlider('fov-slider', 'fov-value', 
    (value) => wasmModule?.set_debug_fov?.(value), 
    (value) => `${value}°`);

  // Mass slider
  setupSlider('mass-slider', 'mass-value', 
    (value) => wasmModule?.set_debug_mass?.(value), 
    (value) => value.toFixed(1));

  // Spin slider
  setupSlider('spin-slider', 'spin-value', 
    (value) => wasmModule?.set_debug_spin?.(value), 
    (value) => value.toFixed(1));

  // Ray steps slider
  setupSlider('ray-steps-slider', 'ray-steps-value', 
    (value) => wasmModule?.set_debug_ray_steps?.(value), 
    (value) => value.toFixed(0));
}

function updateLoadingText(text) {
  const loadingText = document.getElementById('loading-screen')?.querySelector('.loading-text');
  if (loadingText) {
    loadingText.textContent = text;
  }
}

function hideLoadingScreen() {
  const loadingScreen = document.getElementById('loading-screen');
  if (loadingScreen) {
    loadingScreen.style.transition = 'opacity 0.5s ease-out';
    loadingScreen.style.opacity = '0';
    setTimeout(() => {
      loadingScreen.style.display = 'none';
    }, 500);
  }
}

// Make hideLoadingScreen available globally for WASM to call
window.hideLoadingScreen = hideLoadingScreen;

async function main() {
  // Set up canvas sizing before initializing WASM
  setupCanvas();
  setupHelpOverlay();
  
  // Check WebGL support up front to avoid cryptic panics later
  const webgl = checkWebGLSupport();
  if (!webgl.ok) {
    const base = 'Failed to create WebGL context: WebGL is currently disabled or unavailable.';
    const reason = webgl.reason ? ` Details: ${webgl.reason}` : '';
    showFatalError(base + reason);
    return;
  }

  updateLoadingText('Loading WebAssembly module...');
  
  // Wait for the wasm module to be compiled and initialized
  wasmModule = await init();

  updateLoadingText('Setting up graphics context...');
  
  // Set up debug controls now that WASM is loaded
  setupDebugControls();

  // Ensure canvas can receive keyboard events
  const canvas = document.getElementById('wasm-canvas');
  if (canvas) {
    canvas.tabIndex = 0; // Make canvas focusable
    canvas.focus(); // Give canvas initial focus
    
    // Restore focus to canvas when clicked
    canvas.addEventListener('click', () => {
      canvas.focus();
    });
  }

  updateLoadingText('Starting renderer...');

  // Now that initialization is complete, try to start the renderer
  // The WASM renderer will call hideLoadingScreen() when it's ready
  try {
    run();
  } catch (e) {
    // Some WASM/Winit paths intentionally throw an exception for control flow.
    if (isControlFlowException(e)) {
      // Ignore silently; not an actual failure.
      return;
    }
    // Show a friendly message for real failures
    const msg = (e && e.message) ? e.message : String(e);
    showFatalError(`Renderer initialization failed: ${msg}`);
    console.error(e);
  }
}

main().catch((e) => {
  // Filter out benign control-flow exceptions so they don't spam console/UI
  if (isControlFlowException(e)) return;
  console.error(e);
});
