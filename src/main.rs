mod renderer;
mod simulation;

use std::{
    sync::{Arc, RwLock},
    thread,
    time::Instant,
};

use glium::{
    glutin::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    Display,
};
use rand::thread_rng;

use renderer::*;
use simulation::*;

const SIZE_X: f64 = 300.0;
const SIZE_Y: f64 = 300.0;
const VEL_STDEV: f64 = 1.0;

fn main() {
    let mut rng = thread_rng();

    let mut sim = Simulation::new(10000, &mut rng, (SIZE_X, SIZE_Y), VEL_STDEV);
    sim.infect(1, &mut rng);
    let sim_arc = Arc::new(RwLock::new(sim));

    println!("Simulation created.");

    let event_loop = EventLoop::new();

    let wb = WindowBuilder::new().with_title("Pandemic sim");
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let renderer = Renderer::new(&display, SIZE_X / 2.0, SIZE_Y / 2.0, SIZE_X);

    let sim_clone = sim_arc.clone();

    thread::spawn(move || {
        let mut now = Instant::now();
        let mut rng = thread_rng();

        loop {
            let dt = now.elapsed().as_secs_f64();
            now = Instant::now();

            let mut sim = sim_arc.read().unwrap().clone();
            sim.step(dt, &mut rng);
            *sim_arc.write().unwrap() = sim;
        }
    });

    event_loop.run(move |ev, _, control_flow| {
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            _ => (),
        }

        let sim = sim_clone.read().unwrap().clone();
        renderer.draw(&display, &sim);
    });
}
