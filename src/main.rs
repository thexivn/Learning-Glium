extern crate glium;

use glium::{
    glutin::{ContextBuilder, EventsLoop, WindowBuilder, Event, WindowEvent},
    Display,
    Surface
};

fn main() {
    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new().with_title("Titolino");
    let context_builder = ContextBuilder::new();
    let display = Display::new(window_builder, context_builder, &events_loop).unwrap();

    let mut closed = false;
    while !closed {

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.finish().unwrap();
        
        events_loop.poll_events(|ev| {
            match ev {
                Event::WindowEvent {event, ..} => match event {
                    WindowEvent::CloseRequested => closed = true,
                    _ => ()
                },
                _ => (),
            }
        });
    }
}