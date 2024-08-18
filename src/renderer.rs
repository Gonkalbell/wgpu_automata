#[allow(clippy::all)]
mod shaders;

use eframe::{egui_wgpu::CallbackTrait, wgpu};
use egui::Vec2;
use puffin::profile_function;
use wgpu::util::DeviceExt;

use shaders::*;

use crate::app;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

// TODO: While `eframe` does handle a lot of the boilerplate for me, it wasn't really meant for a situation where I am
// mostly doing my own custom rendering. The main challenge is that the only way to do custom rendering is through a
// struct that implements `CallbackTrait`, which I have several nitpicks with:
//   - I don't have direct access to the `wgpu::Surface` or `wgpu::SurfaceTexture`. The `render` function uses the same
//     `wgpu::RenderPass` that the rest of egui uses to render to the surface, but I can't make multiple
//     `wgpu::RenderPass`s that all target the `wgpu::SurfaceTexture`
//   - `CustomCallback` must be recreated every frame. In fact `new_paint_callback` allocates a new Arc every frame.
// If any of these become a deal breaker, I may consider just using `winit` and `egui` directly. .
pub struct RenderCallback {
    pub response: egui::Response,
    pub camera: app::Camera,
}

impl CallbackTrait for RenderCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let rect = self.response.interact_rect;
        let aspect_ratio = rect.aspect_ratio();
        if let Some(renderer) = callback_resources.get::<SceneRenderer>() {
            let camera_data = shaders::textured_quad::Camera {
                origin: (Vec2::new(1., -1.) * self.camera.pos / rect.size()).into(),
                scale: (self.camera.zoom * Vec2::new(1., aspect_ratio)).into(),
            };
            queue.write_buffer(&renderer.camera_buf, 0, bytemuck::bytes_of(&camera_data));
        }
        vec![]
    }

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
}

pub struct SceneRenderer {
    pub camera_buf: wgpu::Buffer,
    camera_bgroup: textured_quad::WgpuBindGroup0,
    // texture_bgroup: textured_quad::WgpuBindGroup0,
    textured_quad_pipeline: wgpu::RenderPipeline,
}

impl SceneRenderer {
    pub fn init(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera"),
            contents: bytemuck::bytes_of(&textured_quad::Camera {
                origin: Vec2::ZERO.into(),
                scale: Vec2::new(9. / 16., 1.).into(),
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bgroup = textured_quad::WgpuBindGroup0::from_bindings(
            device,
            textured_quad::WgpuBindGroup0Entries::new(textured_quad::WgpuBindGroup0EntriesParams {
                res_camera: camera_buf.as_entire_buffer_binding(),
            }),
        );

        let shader = textured_quad::create_shader_module_embed_source(device);
        let textured_quad_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("textured_quad"),
                layout: Some(&textured_quad::create_pipeline_layout(device)),
                vertex: textured_quad::vertex_state(
                    &shader,
                    &textured_quad::vs_textured_quad_entry(),
                ),
                fragment: Some(textured_quad::fragment_state(
                    &shader,
                    &textured_quad::fs_textured_quad_entry([Some(color_format.into())]),
                )),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        Self {
            camera_buf,
            camera_bgroup,
            textured_quad_pipeline,
        }
    }

    pub fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        profile_function!();

        self.camera_bgroup.set(rpass);
        rpass.set_pipeline(&self.textured_quad_pipeline);
        rpass.draw(0..4, 0..1);
    }
}
