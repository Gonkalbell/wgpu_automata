use crate::app;
use eframe::{egui_wgpu::CallbackTrait, wgpu};
use egui::Vec2;
use puffin::profile_function;
use wgpu::util::DeviceExt;

// I previously experimented with using wgsl_bindgen to generate these rust boilerplate for my shader, but there are
// serious limitations with it (and the wgsl_to_wgpu crate). The main limitation is that it generates invalid bind group
// structs for bind groups shared between entry points, compute & vertex shaders, or different shader modules.

const SHADER_SRC: &str = include_str!("automata.wgsl");
const SHADER_DESC: wgpu::ShaderModuleDescriptor = wgpu::ShaderModuleDescriptor {
    label: Some("automata.wgsl"),
    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER_SRC)),
};

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct Camera {
    /// size: 8, offset: 0x0, type: `vec2<f32>`
    pub origin: [f32; 2],
    /// size: 8, offset: 0x8, type: `vec2<f32>`
    pub scale: [f32; 2],
}
unsafe impl bytemuck::Zeroable for Camera {}
unsafe impl bytemuck::Pod for Camera {}

mod res_cur_tex {
    pub const BINDING: u32 = 0;
    pub const LAYOUT_DESC: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: BINDING,
        visibility: wgpu::ShaderStages::all(),
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::ReadOnly,
            format: wgpu::TextureFormat::R32Uint,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    };
}

mod res_next_tex {
    pub const BINDING: u32 = 1;
    pub const LAYOUT_DESC: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: BINDING,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::R32Uint,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    };
}

mod res_camera {
    pub const BINDING: u32 = 0;
    pub const LAYOUT_DESC: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: BINDING,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<super::Camera>() as _),
        },
        count: None,
    };
}

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
    pub settings: app::RenderSettings,
}

impl CallbackTrait for RenderCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(renderer) = callback_resources.get_mut::<SceneRenderer>() {
            let rect = self.response.interact_rect;
            let aspect_ratio = rect.aspect_ratio();

            let camera_data = Camera {
                origin: (Vec2::new(1., -1.) * self.settings.pos / rect.size()).into(),
                scale: (self.settings.zoom * Vec2::new(1., aspect_ratio)).into(),
            };
            queue.write_buffer(&renderer.camera_buf, 0, bytemuck::bytes_of(&camera_data));

            if renderer.texture_size != self.settings.image_size {
                renderer.texture_size = self.settings.image_size;
                renderer.texture_bgroup = create_texture_bgroup(device, renderer.texture_size);
            }
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
    camera_buf: wgpu::Buffer,
    camera_bgroup: wgpu::BindGroup,
    texture_size: [u32; 2],
    texture_bgroup: wgpu::BindGroup,
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
            contents: bytemuck::bytes_of(&Camera {
                origin: Vec2::ZERO.into(),
                scale: Vec2::new(9. / 16., 1.).into(),
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bgroup_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera"),
                entries: &[res_camera::LAYOUT_DESC],
            });
        let camera_bgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera"),
            layout: &camera_bgroup_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: res_camera::BINDING,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        let texture_size = [1, 1];
        let texture_bgroup = create_texture_bgroup(device, texture_size);

        let module = &device.create_shader_module(SHADER_DESC);
        let textured_quad_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("textured_quad"),
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("textured_quad pipeline"),
                        bind_group_layouts: &[
                            &create_texture_bgroup_layout(device),
                            &camera_bgroup_layout,
                        ],
                        push_constant_ranges: &[],
                    }),
                ),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: "vs_textured_quad",
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: "fs_textured_quad",
                    targets: &[Some(color_format.into())],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
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
            texture_size,
            texture_bgroup,
            textured_quad_pipeline,
        }
    }

    pub fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        profile_function!();

        rpass.set_bind_group(0, &self.texture_bgroup, &[]);
        rpass.set_bind_group(1, &self.camera_bgroup, &[]);
        rpass.set_pipeline(&self.textured_quad_pipeline);
        rpass.draw(0..4, 0..1);
    }
}

fn create_texture_bgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Cells"),
        entries: &[res_cur_tex::LAYOUT_DESC, res_next_tex::LAYOUT_DESC],
    })
}

fn create_texture_bgroup(device: &wgpu::Device, texture_size: [u32; 2]) -> wgpu::BindGroup {
    let [width, height] = texture_size;
    let [texture_a, texture_b] = ["CellsA", "CellsB"].map(|label| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Uint,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[wgpu::TextureFormat::R32Uint],
            })
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some(label),
                ..Default::default()
            })
    });

    let bind_group_layout = create_texture_bgroup_layout(device);
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("CellsA -> CellsB"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: res_cur_tex::BINDING,
                resource: wgpu::BindingResource::TextureView(&texture_a),
            },
            wgpu::BindGroupEntry {
                binding: res_next_tex::BINDING,
                resource: wgpu::BindingResource::TextureView(&texture_b),
            },
        ],
    })
}
