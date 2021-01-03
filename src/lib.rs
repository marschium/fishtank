#[macro_use]
extern crate derive_new;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod gui;
use gui::{Gui, SelectedCell};
mod debug;
use debug::DebugInfo;
mod world;
use world::{World, WORLD_WIDTH, WORLD_HEIGHT};
mod cell;
use cell::Cell;

const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 576;


pub fn run() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels + Dear ImGui")
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WORLD_WIDTH as u32, WORLD_HEIGHT as u32, surface_texture)?
    };
    let mut gui = Gui::new(&window, &pixels);
    let mut debug = DebugInfo::new();
    let mut world = World::new();
    let mut selected_cell = SelectedCell::Sand;
    let mut smooth_lighting = false;
    let mut block_spawn = false;

    #[cfg(feature = "web-sys")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        body.append_child(&canvas)
            .expect("Append canvas to HTML body");
    }


    let large_spawn: Vec::<(usize, usize)> = vec! [
           (1,0), (2,0), (3,0), (4,0), (5,0), (7,0), (7,0),
           (1,1), (2,1), (3,1), (4,1), (5,1), (6,1), (7,1),
    (0,2), (1,2), (2,2), (3,2), (4,2), (5,2), (6,2), (7,2), (8,2),
    (0,3), (1,3), (2,3), (3,3), (4,3), (5,3), (6,3), (7,3), (8,3),
    (0,4), (1,4), (2,4), (3,4), (4,4), (5,4), (6,4), (7,4), (8,4),
    (0,5), (1,5), (2,5), (3,5), (4,5), (5,5), (6,5), (7,5), (8,5),
    (0,6), (1,6), (2,6), (3,6), (4,6), (5,6), (6,6), (7,6), (8,6),
           (1,7), (2,7), (3,7), (4,7), (5,7), (6,7), (7,7),
           (1,8), (2,8), (3,8), (4,8), (5,8), (6,8), (7,8),
    ];

    let small_spawn: Vec::<(usize, usize)> = vec! [
           (1,0), (2,0), (3,0), (4,0),
    (0,1), (1,1), (2,1), (3,1), (4,1), (5,1),
    (0,2), (1,2), (2,2), (3,2), (4,2), (5,2),
    (0,3), (1,3), (2,3), (3,3), (4,3), (5,3),
           (1,4), (2,4), (3,4), (4,4),
    ];

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            // Draw the world
            let frame = pixels.get_frame();
            world.draw(frame, smooth_lighting);

            // Prepare Dear ImGui
            gui.prepare(&window).expect("gui.prepare() failed");

            // Render everything together
            let render_result = pixels.render_with(|encoder, render_target, context| {
                // Render the world texture
                context.scaling_renderer.render(encoder, render_target);

                // Render Dear ImGui
                let gui_state = gui.render(&window, encoder, render_target, context, &debug);
                selected_cell = gui_state.selected_cell;
                smooth_lighting = gui_state.smooth_lighting;
                block_spawn = gui_state.block_spawn;
            });

            // Basic error handling
            if render_result
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        gui.handle_event(&window, &event);
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            let mut mouse_pos = None;
            if let Some(mpos) = input.mouse() {
                mouse_pos = pixels.window_pos_to_pixel(mpos).ok();
            }
            debug.world_pos = mouse_pos;

            debug.spawning = false;
            if input.mouse_held(0) && !block_spawn {
                debug.spawning = true;
                match mouse_pos {
                    Some(pos) => {
                        match selected_cell {
                            SelectedCell::Sand => {
                                for d in &large_spawn {
                                    world.spawn((pos.0 + d.0, pos.1 + d.1), Cell::Sand);
                                }
                            },
                            SelectedCell::Stone => {
                                for d in &small_spawn {
                                    world.spawn((pos.0 + d.0, pos.1 + d.1), Cell::Stone); 
                                }
                            },
                            SelectedCell::Fizzer => world.spawn((pos.0, pos.1), Cell::Fizzer),
                            SelectedCell::BottomFeeder => world.spawn((pos.0, pos.1), Cell::BottomFeeder),
                            SelectedCell::Seed => {
                                for d in &small_spawn {
                                    world.spawn((pos.0 + d.0, pos.1 + d.1), Cell::Seed); 
                                }
                            },
                            SelectedCell::KelpSeed => world.spawn((pos.0, pos.1), Cell::KelpSeed),
                            SelectedCell::Fish => world.spawn((pos.0, pos.1), Cell::new_fish()),
                            SelectedCell::Algae => world.spawn((pos.0, pos.1), Cell::new_algae()),
                            SelectedCell::Worm => world.spawn((pos.0, pos.1), Cell::new_worm()),
                        };
                    }
                    None => {}
                }
            }
            else if input.mouse_held(1) && !block_spawn {
                debug.spawning = true;
                match mouse_pos {
                    Some(pos) => {
                        for d in &small_spawn {
                            world.spawn((pos.0 + d.0, pos.1 + d.1), Cell::Water); 
                        }
                    }
                    _ => {
                    }
                }
            }
            
            world.update();
            window.request_redraw();
        }
    });
}

#[cfg(feature = "web-sys")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn run() {
        console_log::init_with_level(log::Level::Debug);

        super::run();
    }
}