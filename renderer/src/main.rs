use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    println!("{}", simulation::get_placeholder_string());

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Black Hole Simulator")
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                elwt.exit();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // This is where rendering logic using wgpu will go.
            }
            _ => (),
        }
    }).unwrap();
}
