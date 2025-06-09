use std::sync::{
    Arc,
    Mutex,
};
use std::thread::JoinHandle;
use std::time::{
    Duration,
    Instant,
};

use cgmath::{
    Point2,
    Vector2,
    Zero,
};
use pipeline::Pipeline;
use rand::distr::{
    Distribution,
    Uniform,
};
use rand::rng;
use rand_distr::Normal;
use rendering::RenderState;
use simulation::quadtree::Quadtree;
use simulation::{
    Body,
    Simulation,
};
use wgpu::{
    Color,
    SurfaceError,
};
use winit::dpi::{
    PhysicalSize,
    Size,
};
use winit::event::{
    ElementState,
    Event,
    KeyEvent,
    WindowEvent,
};
use winit::event_loop::EventLoop;
use winit::keyboard::{
    KeyCode,
    PhysicalKey,
};
use winit::window::{
    Window,
    WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::rendering::rgb;

mod pipeline;
mod rendering;
mod simulation;
mod utility;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Trace).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let num_bodies = 1000000;

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut pipeline = Pipeline::new(&window).await;
    let simulation = Arc::new(Mutex::new(Simulation::new(
        std::iter::repeat_with(|| {
            // let pos_dist = Uniform::new(-0.5, 0.5).unwrap();
            let pos_dist = Normal::new(0.0, 0.5).unwrap();
            let vel_dist = Normal::new(0.0, 0.0001).unwrap();

            let pos_x: f32 = pos_dist.sample(&mut rng());
            let pos_y: f32 = pos_dist.sample(&mut rng());

            let vel_x: f32 = vel_dist.sample(&mut rng());
            let vel_y: f32 = vel_dist.sample(&mut rng());

            Body {
                position: Point2::new(pos_x, pos_y),
                velocity: Vector2::new(vel_x, vel_y),
                mass: 1.0,
                radius: 0.005,
                color: rgb(0xC4, 0x60, 0x3B),
            }
        })
        .take(num_bodies),
        0.5,
    )));

    // two threads with the simulation as shared state:
    // 1. simulation
    // 2. rendering

    let simulation_thread = {
        let simulation = simulation.clone();

        std::thread::spawn(move || {
            let mut previous_time = Instant::now();

            loop {
                let current_time = Instant::now();
                let dt = current_time - previous_time;
                previous_time = current_time;

                simulation.lock().unwrap().advance(dt).unwrap();

                std::thread::sleep(Duration::from_millis(10));
            }
        })
    };

    let mut render_state = RenderState::new(&pipeline.device, num_bodies);

    log::info!("Created window and event loop! Window inner size: {:?}", window.inner_size());

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("graviton-canvas")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut surface_configured = false;

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent { window_id, ref event } if window_id == pipeline.window.id() => match event {
                WindowEvent::RedrawRequested => {
                    pipeline.window.request_redraw();

                    if !surface_configured {
                        return;
                    }

                    let simulation = simulation.lock().unwrap();
                    match render_state.render(&mut pipeline, &simulation) {
                        Ok(_) => {}
                        Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                            pipeline.resize(pipeline.size);
                        }
                        Err(SurfaceError::OutOfMemory | SurfaceError::Other) => {
                            log::error!("Surface out of memory");
                            control_flow.exit();
                        }
                        Err(SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }
                    }
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => control_flow.exit(),
                WindowEvent::Resized(physical_size) => {
                    pipeline.resize(*physical_size);
                    surface_configured = true;
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(keycode),
                            ..
                        },
                    ..
                } => {
                    match keycode {
                        KeyCode::KeyG => {
                            // toggle tree drawing
                            render_state.settings_mut().toggle_draw_tree();
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}
