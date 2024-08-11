mod camera;
mod shaders;

use eframe::wgpu;
use puffin::profile_function;
use wgpu::util::DeviceExt;

use camera::ArcBallCamera;

use shaders::*;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn create_vertices() -> (Vec<cube::Vertex>, Vec<u16>) {
    use cube::Vertex;

    let vertex_data = [
        // top (0, 0, 1)
        Vertex::new([-1., -1., 1., 1.].into(), [0., 0.].into()),
        Vertex::new([1., -1., 1., 1.].into(), [1., 0.].into()),
        Vertex::new([1., 1., 1., 1.].into(), [1., 1.].into()),
        Vertex::new([-1., 1., 1., 1.].into(), [0., 1.].into()),
        // bottom (0, 0, -1)
        Vertex::new([-1., 1., -1., 1.].into(), [1., 0.].into()),
        Vertex::new([1., 1., -1., 1.].into(), [0., 0.].into()),
        Vertex::new([1., -1., -1., 1.].into(), [0., 1.].into()),
        Vertex::new([-1., -1., -1., 1.].into(), [1., 1.].into()),
        // right (1, 0, 0)
        Vertex::new([1., -1., -1., 1.].into(), [0., 0.].into()),
        Vertex::new([1., 1., -1., 1.].into(), [1., 0.].into()),
        Vertex::new([1., 1., 1., 1.].into(), [1., 1.].into()),
        Vertex::new([1., -1., 1., 1.].into(), [0., 1.].into()),
        // left (-1, 0, 0)
        Vertex::new([-1., -1., 1., 1.].into(), [1., 0.].into()),
        Vertex::new([-1., 1., 1., 1.].into(), [0., 0.].into()),
        Vertex::new([-1., 1., -1., 1.].into(), [0., 1.].into()),
        Vertex::new([-1., -1., -1., 1.].into(), [1., 1.].into()),
        // front (0, 1, 0)
        Vertex::new([1., 1., -1., 1.].into(), [1., 0.].into()),
        Vertex::new([-1., 1., -1., 1.].into(), [0., 0.].into()),
        Vertex::new([-1., 1., 1., 1.].into(), [0., 1.].into()),
        Vertex::new([1., 1., 1., 1.].into(), [1., 1.].into()),
        // back (0, -1, 0)
        Vertex::new([1., -1., 1., 1.].into(), [0., 0.].into()),
        Vertex::new([-1., -1., 1., 1.].into(), [1., 0.].into()),
        Vertex::new([-1., -1., -1., 1.].into(), [1., 1.].into()),
        Vertex::new([1., -1., -1., 1.].into(), [0., 1.].into()),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_texels(size: usize) -> Vec<u8> {
    (0..size * size)
        .map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            count
        })
        .collect()
}

type CameraBindGroup = shaders::cube::WgpuBindGroup0;
type MaterialBindGroup = shaders::cube::WgpuBindGroup1;
type SkyboxBindGroup = shaders::skybox::WgpuBindGroup1;

pub struct SceneRenderer {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    camera_buf: wgpu::Buffer,
    user_camera: ArcBallCamera,

    // BIND GROUPS
    _empty_bgroup: wgpu::BindGroup,
    camera_bgroup: CameraBindGroup,
    material_bgroup: MaterialBindGroup,
    skybox_bgroup: SkyboxBindGroup,

    // PIPELINES
    cube_pipeline: wgpu::RenderPipeline,
    wireframe_pipeline: Option<wgpu::RenderPipeline>,
    skybox_pipeline: wgpu::RenderPipeline,
}

impl SceneRenderer {
    pub fn init(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        // Create the vertex and index buffers
        let (vertex_data, index_data) = create_vertices();

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create the texture
        let size = 256u32;
        let texels = create_texels(size as usize);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };
        let cube_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let cube_tview = cube_tex.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            cube_tex.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size),
                rows_per_image: None,
            },
            texture_extent,
        );

        let user_camera = ArcBallCamera::default();

        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::bytes_of(&shaders::camera::Camera {
                view: Default::default(),
                view_inv: Default::default(),
                proj: Default::default(),
                proj_inv: Default::default(),
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let ktx_reader = ktx2::Reader::new(include_bytes!("../assets/rgba8.ktx2")).unwrap();
        let mut image = Vec::with_capacity(ktx_reader.data().len());
        for level in ktx_reader.levels() {
            image.extend_from_slice(level);
        }
        let ktx_header = ktx_reader.header();
        let skybox_tex = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("../assets/rgba8.ktx2"),
                size: wgpu::Extent3d {
                    width: ktx_header.pixel_width,
                    height: ktx_header.pixel_height,
                    depth_or_array_layers: ktx_header.face_count,
                },
                mip_level_count: ktx_header.level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::MipMajor,
            &image,
        );
        let skybox_tview = skybox_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("../assets/rgba8.ktx2"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..wgpu::TextureViewDescriptor::default()
        });

        // Create bind groups

        let empty_bgroup_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("empty"),
                entries: &[],
            });
        let empty_bgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("empty"),
            layout: &empty_bgroup_layout,
            entries: &[],
        });

        let camera_bgroup = CameraBindGroup::from_bindings(
            device,
            cube::WgpuBindGroup0Entries::new(cube::WgpuBindGroup0EntriesParams {
                res_camera: camera_buf.as_entire_buffer_binding(),
            }),
        );

        let material_bgroup = MaterialBindGroup::from_bindings(
            device,
            cube::WgpuBindGroup1Entries::new(cube::WgpuBindGroup1EntriesParams {
                res_color: &cube_tview,
            }),
        );

        let skybox_bgroup = SkyboxBindGroup::from_bindings(
            device,
            skybox::WgpuBindGroup1Entries::new(skybox::WgpuBindGroup1EntriesParams {
                res_texture: &skybox_tview,
                res_sampler: &device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("skybox sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                }),
            }),
        );

        // Create pipelines

        let shader = cube::create_shader_module_embed_source(&device);
        let cube_pipeline_layout = cube::create_pipeline_layout(&device);
        let cube_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube_pipeline"),
            layout: Some(&cube_pipeline_layout),
            vertex: cube::vertex_state(&shader, &cube::vs_mesh_entry(wgpu::VertexStepMode::Vertex)),
            fragment: Some(cube::fragment_state(
                &shader,
                &cube::fs_mesh_entry([Some(color_format.into())]),
            )),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let wireframe_pipeline = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            Some(
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("wireframe_pipeline"),
                    layout: Some(&cube_pipeline_layout),
                    vertex: cube::vertex_state(
                        &shader,
                        &cube::vs_mesh_entry(wgpu::VertexStepMode::Vertex),
                    ),
                    fragment: Some(cube::fragment_state(
                        &shader,
                        &cube::fs_wireframe_entry([Some(wgpu::ColorTargetState {
                            format: color_format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    operation: wgpu::BlendOperation::Add,
                                    src_factor: wgpu::BlendFactor::SrcAlpha,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                },
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })]),
                    )),
                    primitive: wgpu::PrimitiveState {
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Line,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                }),
            )
        } else {
            None
        };

        let shader = skybox::create_shader_module_embed_source(&device);
        let skybox_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("skybox"),
            layout: Some(&skybox::create_pipeline_layout(device)),
            vertex: skybox::vertex_state(&shader, &skybox::vs_skybox_entry()),
            fragment: Some(skybox::fragment_state(
                &shader,
                &skybox::fs_skybox_entry([Some(color_format.into())]),
            )),
            primitive: wgpu::PrimitiveState {
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
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            user_camera,
            camera_buf,

            _empty_bgroup: empty_bgroup,
            camera_bgroup,
            material_bgroup,
            skybox_bgroup,

            cube_pipeline,
            wireframe_pipeline,
            skybox_pipeline,
        }
    }

    pub fn prepare(
        &self,
        _device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
    ) -> Option<wgpu::CommandBuffer> {
        profile_function!();

        let view = self.user_camera.view_matrix();
        let proj = self.user_camera.projection_matrix();
        let camera = shaders::camera::Camera {
            view,
            view_inv: view.inverse(),
            proj,
            proj_inv: proj.inverse(),
        };
        queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&camera));

        None
    }

    pub fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        profile_function!();

        self.camera_bgroup.set(rpass);

        self.material_bgroup.set(rpass);
        rpass.set_pipeline(&self.cube_pipeline);
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        if let Some(ref pipe) = self.wireframe_pipeline {
            rpass.set_pipeline(pipe);
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        self.skybox_bgroup.set(rpass);
        rpass.set_pipeline(&self.skybox_pipeline);
        rpass.draw(0..3, 0..1);
    }

    pub fn run_ui(&mut self, ctx: &egui::Context) {
        profile_function!();

        if !ctx.wants_keyboard_input() && !ctx.wants_pointer_input() {
            ctx.input(|input| {
                self.user_camera.update(input);
            });
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                }

                let mut are_scopes_on = puffin::are_scopes_on();
                ui.toggle_value(&mut are_scopes_on, "Profiler");
                puffin::set_scopes_on(are_scopes_on);

                ui.menu_button("Camera", |ui| self.user_camera.run_ui(ui));
            });

            puffin_egui::show_viewport_if_enabled(ctx);
        });
    }
}
