mod renderer;
mod simulation;

use std::{
    fs::File,
    io::Read,
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

fn main() {
    let mut rng = thread_rng();

    let mut conf_file = File::open("config.toml").unwrap();
    let mut conf_str = String::new();
    conf_file.read_to_string(&mut conf_str).unwrap();
    let params = toml::from_str(&conf_str).unwrap();

    let mut sim = Simulation::new(&mut rng, params);
    sim.infect(params.init_infected, &mut rng);
    sim.vaccinate(params.init_vaccinated, &mut rng);
    let sim_arc = Arc::new(RwLock::new(sim));

    println!("Simulation created.");

    let event_loop = EventLoop::new();

    let wb = WindowBuilder::new().with_title("Pandemic sim");
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let mut renderer = Renderer::new(
        &display,
        params.size_x / 2.0,
        params.size_y / 2.0,
        params.size_x,
    );

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
