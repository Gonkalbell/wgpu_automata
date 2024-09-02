// Flocking boids example with gpu compute update pass
// adapted from https://github.com/austinEng/webgpu-samples/blob/master/src/examples/computeBoids.ts

use crate::shaders::*;
use eframe::egui_wgpu::CallbackTrait;
use nanorand::{Rng, WyRand};
use wgpu::util::DeviceExt;

// number of boid particles to simulate

const NUM_PARTICLES: u32 = 12000;

// number of single-particle calculations (invocations) in each gpu work group

const PARTICLES_PER_GROUP: u32 = 64;

/// Persistent WGPU data for particle rendering and simulation
pub struct ParticleSystem {
    particle_bind_groups: Vec<wgpu::BindGroup>,
    particle_buffers: Vec<wgpu::Buffer>,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    frame_num: usize,
}

impl ParticleSystem {
    /// constructs initial instance of Example struct
    pub fn init(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let shader = boids::create_shader_module(device);

        // buffer for simulation parameters uniform

        let sim_param_data = boids::SimParams {
            delta_time: 0.04f32,
            separation_distance: 0.025,
            separation_scale: 0.05,
            alignment_distance: 0.025,
            alignment_scale: 0.005,
            cohesion_distance: 0.1,
            cohesion_scale: 0.02,
        };
        let sim_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameter Buffer"),
            contents: bytemuck::bytes_of(&sim_param_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // create compute bind layout group and compute pipeline layout

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ..boids::params::LAYOUT
                    },
                    wgpu::BindGroupLayoutEntry {
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ..boids::particles_src::LAYOUT
                    },
                    wgpu::BindGroupLayoutEntry {
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ..boids::particles_dst::LAYOUT
                    },
                ],
                label: None,
            });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        // create render pipeline with empty bind group layout

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: boids::vertex_state(
                &shader,
                &boids::boids_vs_entry(wgpu::VertexStepMode::Instance),
            ),
            fragment: Some(boids::fragment_state(
                &shader,
                &boids::boids_fs_entry([Some(color_format.into())]),
            )),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // create compute pipeline

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: boids::ENTRY_BOIDS_CS,
            compilation_options: Default::default(),
        });

        // buffer for all particles

        let mut rng = WyRand::new_seed(42);
        let mut unif = || rng.generate::<f32>() * 2f32 - 1f32; // Generate a num (-1, 1)
        let initial_particle_data: Vec<_> = (0..NUM_PARTICLES)
            .map(|_| boids::Particle {
                pos: [unif(), unif()],
                vel: [unif(), unif()],
            })
            .collect();

        // creates two buffers of particle data each of size NUM_PARTICLES
        // the two buffers alternate as dst and src for each frame

        let mut particle_buffers = Vec::<wgpu::Buffer>::new();
        let mut particle_bind_groups = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            particle_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Particle Buffer {i}")),
                    contents: bytemuck::cast_slice(&initial_particle_data),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }

        // create two bind groups, one for each buffer as the src
        // where the alternate buffer is used as the dst

        for i in 0..2 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    boids::params::bind_group_entry(sim_param_buffer.as_entire_buffer_binding()),
                    boids::particles_src::bind_group_entry(
                        particle_buffers[i].as_entire_buffer_binding(),
                    ),
                    boids::particles_dst::bind_group_entry(
                        particle_buffers[(i + 1) % 2].as_entire_buffer_binding(),
                    ),
                ],
                label: None,
            }));
        }

        // returns Example struct and No encoder commands

        ParticleSystem {
            particle_bind_groups,
            particle_buffers,
            compute_pipeline,
            render_pipeline,
            frame_num: 0,
        }
    }
}

// TODO: While `eframe` does handle a lot of the boilerplate for me, it wasn't really meant for a situation where I am
// mostly doing my own custom rendering. The main challenge is that the only way to do custom rendering is through a
// struct that implements `CallbackTrait`, which I have several nitpicks with:
//   - I don't have direct access to the `wgpu::Surface` or `wgpu::SurfaceTexture`. The `render` function uses the same
//     `wgpu::RenderPass` that the rest of egui uses to render to the surface, but I can't make multiple
//     `wgpu::RenderPass`s that all target the `wgpu::SurfaceTexture`
//   - `CustomCallback` must be recreated every frame. In fact `new_paint_callback` allocates a new Arc every frame.
// If any of these become a deal breaker, I may consider just using `winit` and `egui` directly. .
pub struct RenderCallback {
    pub sim_params: boids::SimParams,
    pub num_sim_updates: u32,
}

impl CallbackTrait for RenderCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        command_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(renderer) = callback_resources.get_mut::<ParticleSystem>() {
            command_encoder.push_debug_group("compute boid movement");
            {
                // compute pass
                let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&renderer.compute_pipeline);
                // calculates number of work groups from PARTICLES_PER_GROUP constant
                let work_group_count =
                    ((NUM_PARTICLES as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;
                for _ in 0..self.num_sim_updates {
                    cpass.set_bind_group(
                        0,
                        &renderer.particle_bind_groups[renderer.frame_num % 2],
                        &[],
                    );
                    cpass.dispatch_workgroups(work_group_count, 1, 1);
                    renderer.frame_num += 1;
                }
            }
            command_encoder.pop_debug_group();
        }
        vec![]
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        rpass: &mut eframe::wgpu::RenderPass<'a>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        if let Some(renderer) = callback_resources.get::<ParticleSystem>() {
            rpass.set_pipeline(&renderer.render_pipeline);
            // render dst particles
            rpass.set_vertex_buffer(
                0,
                renderer.particle_buffers[(renderer.frame_num + 1) % 2].slice(..),
            );
            // the three instance-local vertices
            rpass.draw(0..3, 0..NUM_PARTICLES);
        }
    }
}
