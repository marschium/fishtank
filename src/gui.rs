use pixels::{wgpu, PixelsContext};
use std::time::Instant;

use crate::debug::DebugInfo;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SelectedCell {
    Sand,
    Seed,
    Fish,
    BottomFeeder,
    Algae,
    Stone,
    Fizzer,
    KelpSeed,
    Worm
}

pub(crate) struct GuiState {
    pub selected_cell : SelectedCell,
    pub smooth_lighting : bool,
    pub block_spawn: bool
}

/// Manages all state required for rendering Dear ImGui over `Pixels`.
pub(crate) struct Gui {
    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    last_frame: Instant,
    last_cursor: Option<imgui::MouseCursor>,
    selected_cell : SelectedCell,
    smooth_lighting: bool
}

impl Gui {
    /// Create Dear ImGui.
    pub(crate) fn new(window: &winit::window::Window, pixels: &pixels::Pixels::<winit::window::Window>) -> Self {
        // Create Dear ImGui context
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        // Initialize winit platform support
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );

        // Configure Dear ImGui fonts
        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        // Fix incorrect colors with sRGB framebuffer
        let style = imgui.style_mut();
        for color in 0..style.colors.len() {
            style.colors[color] = gamma_to_linear(style.colors[color]);
        }

        // Create Dear ImGui WGPU renderer
        let device = pixels.device();
        let queue = pixels.queue();
        let texture_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let renderer = imgui_wgpu::Renderer::new(&mut imgui, &device, &queue, texture_format);

        // Return GUI context
        Self {
            imgui,
            platform,
            renderer,
            last_frame: Instant::now(),
            last_cursor: None,
            selected_cell: SelectedCell::Sand,
            smooth_lighting: false
        }
    }

    /// Prepare Dear ImGui.
    pub(crate) fn prepare(
        &mut self,
        window: &winit::window::Window,
    ) -> Result<(), winit::error::ExternalError> {
        // Prepare Dear ImGui
        let io = self.imgui.io_mut();
        self.last_frame = io.update_delta_time(self.last_frame);
        self.platform.prepare_frame(io, window)
    }

    /// Render Dear ImGui.
    pub(crate) fn render(
        &mut self,
        window: &winit::window::Window,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
        debug: &DebugInfo
    ) -> GuiState {
        // Start a new Dear ImGui frame and update the cursor
        let ui = self.imgui.frame();

        let mouse_cursor = ui.mouse_cursor();
        if self.last_cursor != mouse_cursor {
            self.last_cursor = mouse_cursor;
            self.platform.prepare_render(&ui, window);        
        }

        let mut selected_cell = self.selected_cell;
        let mut smooth_lighting = self.smooth_lighting;
        let mut block_spawn = false;
        imgui::Window::new(imgui::im_str!("Debug"))
            .position([50.0, 50.0], imgui::Condition::FirstUseEver)
            .size([200.0, 200.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(imgui::im_str!("LMB = Spawn"));
                ui.text(imgui::im_str!("RMB = Clear"));     
                ui.text(format!("{:.2} FPS", ui.io().framerate));                      
                ui.text(format!("Spawning: {}", debug.spawning));
                ui.text(format!(
                    "World Position: ({:.1},{:.1})", debug.world_pos.unwrap_or_default().0, debug.world_pos.unwrap_or_default().1
                ));
                block_spawn |= ui.checkbox(imgui::im_str!("Smooth Lighting"), &mut smooth_lighting); 
                block_spawn |= ui.is_window_hovered();        
            });

        
        imgui::Window::new(imgui::im_str!("Cells"))
            .position([50.0, 260.0], imgui::Condition::FirstUseEver)
            .size([150.0, 300.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                let mut cell_button = |name, selected, tooltip| {
                    block_spawn |= ui.radio_button(name, &mut selected_cell, selected);
                    block_spawn |= ui.is_item_hovered();
                    if ui.is_item_hovered() {
                        ui.tooltip_text(tooltip);
                    }
                };
                cell_button(imgui::im_str!("Sand"), SelectedCell::Sand, "Falls to the ground.");
                cell_button(imgui::im_str!("Plant"), SelectedCell::Seed, "Grows. Dies without light.");
                cell_button(imgui::im_str!("Fish"), SelectedCell::Fish, "Eats algae and worms.");
                cell_button(imgui::im_str!("Bacteria"), SelectedCell::BottomFeeder, "Eats waste. Makes nitrogen.");
                cell_button(imgui::im_str!("Algae"), SelectedCell::Algae, "Eats nitrogen.");
                cell_button(imgui::im_str!("Stone"), SelectedCell::Stone, "Blocks light.");
                cell_button(imgui::im_str!("Fizzer"), SelectedCell::Fizzer, "Makes bubbles.");
                cell_button(imgui::im_str!("Kelp"), SelectedCell::KelpSeed, "Grows. Dies without light.");       
                cell_button(imgui::im_str!("Worm"), SelectedCell::Worm, "Eats algae and waste. Grows.");           
                block_spawn |= ui.is_window_hovered(); 
            });

        // Render Dear ImGui with WGPU
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: render_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.selected_cell = selected_cell;
        self.smooth_lighting = smooth_lighting;
        let _ = self.renderer.render(ui.render(), &context.queue, &context.device, &mut rpass);
        GuiState {
            selected_cell,
            smooth_lighting,
            block_spawn
        }
    }

    /// Handle any outstanding events.
    pub(crate) fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<()>,
    ) {
        self.platform
            .handle_event(self.imgui.io_mut(), window, event);
    }
}

fn gamma_to_linear(color: [f32; 4]) -> [f32; 4] {
    const GAMMA: f32 = 2.2;

    let x = color[0].powf(GAMMA);
    let y = color[1].powf(GAMMA);
    let z = color[2].powf(GAMMA);
    let w = 1.0 - (1.0 - color[3]).powf(GAMMA);

    [x, y, z, w]
}
