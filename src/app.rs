//! Since I mostly want to do my own rendering, very little actually happens in the top level `App` struct. Instead,
//! most of the rendering logic actually happens in `renderer.rs`

mod particles;
mod profiler;

use std::sync::LazyLock;

use egui::{mutex::Mutex, Vec2, Widget};
use puffin::{profile_function, FrameView, GlobalFrameView, GlobalProfiler};
use puffin_egui::{MaybeMutRef, ProfilerUi};

use crate::shaders::boids;
use particles::{ParticleSystem, RenderCallback};

pub static PUFFIN_GPU_FRAME_VIEW: LazyLock<Mutex<FrameView>> = LazyLock::new(|| Default::default());

pub static PUFFIN_GPU_PROFILER: LazyLock<Mutex<GlobalProfiler>> = LazyLock::new(|| {
    let mut profiler = GlobalProfiler::default();
    let _sink_id = profiler.add_sink(Box::new(move |frame| {
        PUFFIN_GPU_FRAME_VIEW.lock().add_frame(frame);
    }));
    // GlobalFrameView might be created after scope scopes were already created
    // and our registered sink won't see them without prior propagation.
    profiler.emit_scope_snapshot();

    Mutex::new(profiler)
});

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RendererApp {
    is_playing: bool,
    sim_delta_time: f32,
    sim_speed: f32,
    leftover_sim_frames: f32,
    num_particles: u32,

    show_cpu_profiler: bool,
    show_gpu_profiler: bool,
    #[serde(skip)]
    gpu_profiler_ui: ProfilerUi,
}

impl Default for RendererApp {
    fn default() -> Self {
        Self {
            is_playing: true,
            sim_delta_time: 0.04,
            sim_speed: 1.,
            leftover_sim_frames: 0.,
            num_particles: 10000,

            show_cpu_profiler: false,
            show_gpu_profiler: false,
            gpu_profiler_ui: Default::default(),
        }
    }
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
            .insert(ParticleSystem::init(
                &wgpu_render_state.device,
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    ui.toggle_value(&mut self.show_cpu_profiler, "CPU Profiler");
                }

                ui.toggle_value(&mut self.show_gpu_profiler, "GPU Profiler");
                puffin::set_scopes_on(self.show_cpu_profiler || self.show_gpu_profiler);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("reset UI").clicked() {
                        *self = Default::default();
                        ui.ctx().memory_mut(|w| *w = Default::default());
                    }
                });
            });

            if self.show_cpu_profiler {
                puffin_egui::profiler_window(ctx);
            }
            if self.show_gpu_profiler {
                let mut frame_view = PUFFIN_GPU_FRAME_VIEW.lock();
                egui::Window::new("GPU Profiler").show(ctx, |ui| {
                    self.gpu_profiler_ui
                        .ui(ui, &mut MaybeMutRef::MutRef(&mut frame_view))
                });
            }
        });

        let mut single_step = false;
        egui::SidePanel::right("Settings").show(ctx, |ui| {
            ui.toggle_value(&mut self.is_playing, "Play");
            if !self.is_playing {
                single_step = ui.button("Step").clicked();
            }
            egui::Slider::new(&mut self.sim_delta_time, 0.004..=0.1)
                .text("Simulation Delta Time (s)")
                .ui(ui);
            egui::Slider::new(&mut self.sim_speed, 0. ..=10.)
                .text("Simulation Speed Multiplier")
                .ui(ui);
            egui::Slider::new(&mut self.num_particles, 0..=particles::MAX_PARTICLES as u32)
                .text("Number of Boids")
                .ui(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let min_size = ui.available_size().min_elem();
                let (rect, _response) =
                    ui.allocate_exact_size(Vec2::splat(min_size), egui::Sense::click_and_drag());

                let num_sim_updates = if self.is_playing {
                    let render_dt = ui.ctx().input(|input| input.stable_dt);
                    let sim_frames = self.leftover_sim_frames + render_dt / self.sim_delta_time;
                    self.leftover_sim_frames = sim_frames.fract();
                    sim_frames as u32
                } else if single_step {
                    1
                } else {
                    0
                };
                let sim_params = boids::SimParams {
                    num_particles: self.num_particles,
                    delta_time: self.sim_delta_time * self.sim_speed,
                    separation_distance: 0.025,
                    separation_scale: 0.05,
                    alignment_distance: 0.025,
                    alignment_scale: 0.005,
                    cohesion_distance: 0.1,
                    cohesion_scale: 0.02,
                };

                ui.painter()
                    .add(eframe::egui_wgpu::Callback::new_paint_callback(
                        rect,
                        RenderCallback {
                            sim_params,
                            num_sim_updates,
                        },
                    ));
            });
        });

        if self.is_playing {
            ctx.request_repaint();
        }

        PUFFIN_GPU_PROFILER.lock().new_frame();
    }
}
