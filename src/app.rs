use eframe::egui_wgpu::CallbackTrait;
use egui::LayerId;

use crate::renderer::SceneRenderer;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

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
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(renderer) = frame
            .wgpu_render_state()
            .expect("WGPU is not properly initialized")
            .renderer
            .write()
            .callback_resources
            .get_mut::<SceneRenderer>()
        {
            renderer.run_ui(ctx);
        }

        ctx.layer_painter(LayerId::background()).add(
            eframe::egui_wgpu::Callback::new_paint_callback(ctx.available_rect(), CustomCallback),
        );
    }
}

struct CustomCallback;

impl CallbackTrait for CustomCallback {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'a>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        if let Some(renderer) = callback_resources.get::<SceneRenderer>() {
            renderer.render(render_pass);
        }
    }

    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        if let Some(renderer) = callback_resources.get::<SceneRenderer>() {
            return Vec::from_iter(renderer.prepare(
                device,
                queue,
                screen_descriptor,
                egui_encoder,
            ));
        }
        Vec::new()
    }
}
