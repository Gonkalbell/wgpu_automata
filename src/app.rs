//! Since I mostly want to do my own rendering, very little actually happens in the top level `App` struct. Instead,
//! most of the rendering logic actually happens in `renderer.rs`

mod renderer;

use egui::{CollapsingHeader, Vec2};
use puffin::profile_function;

use renderer::{RenderCallback, SceneRenderer};
use web_time::Duration;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RenderSettings {
    pub pos: Vec2,
    pub zoom: f32,

    pub image_size: [u32; 2],

    pub playing: bool,
    pub update_rate: Duration,
}

impl Default for RenderSettings {
    fn default() -> Self {
        RenderSettings {
            pos: Vec2::default(),
            zoom: 1.,
            image_size: [100, 100],
            playing: true,
            update_rate: Duration::from_secs_f32(0.1),
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RendererApp {
    settings: RenderSettings,
}

impl RendererApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize the renderer
        let wgpu_render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("WGPU is not properly initialized");
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(SceneRenderer::init(
                &wgpu_render_state.device,
                &wgpu_render_state.queue,
                wgpu_render_state.target_format,
            ));

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for RendererApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(_renderer) = frame
            .wgpu_render_state()
            .expect("WGPU is not properly initialized")
            .renderer
            .write()
            .callback_resources
            .get_mut::<SceneRenderer>()
        {
            profile_function!();

            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.menu_button("File", |ui| {
                            if ui.button("Quit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });

                        // eframe doesn't support puffin on browser because it might not have a high resolution clock.
                        let mut are_scopes_on = puffin::are_scopes_on();
                        ui.toggle_value(&mut are_scopes_on, "Profiler");
                        puffin::set_scopes_on(are_scopes_on);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("reset UI").clicked() {
                                *self = Default::default();
                                ui.ctx().memory_mut(|w| *w = Default::default());
                            }
                        });
                    }
                });

                puffin_egui::show_viewport_if_enabled(ctx);
            });
        };

        egui::SidePanel::left("Settings").show(ctx, |ui| {
            CollapsingHeader::new("Camera")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("position:");
                        ui.add(egui::DragValue::new(&mut self.settings.pos.x));
                        ui.add(egui::DragValue::new(&mut self.settings.pos.y));
                    });

                    ui.horizontal(|ui| {
                        ui.label("zoom:");
                        ui.add(egui::DragValue::new(&mut self.settings.zoom).speed(0.02));
                    });
                });
            CollapsingHeader::new("Cells")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("size:");
                        ui.add(egui::DragValue::new(&mut self.settings.image_size[0]));
                        ui.add(egui::DragValue::new(&mut self.settings.image_size[1]));
                    });
                });
            CollapsingHeader::new("Simulation")
                .default_open(true)
                .show(ui, |ui| {
                    ui.toggle_value(&mut self.settings.playing, "Play");
                    if self.settings.playing {
                        ui.ctx().request_repaint();
                    }
                });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

                if response.dragged_by(egui::PointerButton::Secondary) {
                    self.settings.pos += response.drag_delta();
                }
                ui.ctx().input(|input| {
                    self.settings.zoom *= input.zoom_delta();
                });

                ui.painter()
                    .add(eframe::egui_wgpu::Callback::new_paint_callback(
                        rect,
                        RenderCallback {
                            response,
                            settings: self.settings.clone(),
                        },
                    ));
            });
        });
    }
}
