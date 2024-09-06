#[repr(C)]
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    bytemuck :: Pod,
    bytemuck :: Zeroable,
    serde :: Serialize,
    serde :: Deserialize,
)]
pub struct Particle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
}
const _: () = assert!(
    std::mem::size_of::<Particle>() == 16,
    "size of Particle does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(Particle, pos) == 0,
    "offset of Particle.pos does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(Particle, vel) == 8,
    "offset of Particle.vel does not match WGSL"
);
#[repr(C)]
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    bytemuck :: Pod,
    bytemuck :: Zeroable,
    serde :: Serialize,
    serde :: Deserialize,
)]
pub struct SimParams {
    pub num_particles: u32,
    pub delta_time: f32,
    pub separation_distance: f32,
    pub alignment_distance: f32,
    pub cohesion_distance: f32,
    pub separation_scale: f32,
    pub alignment_scale: f32,
    pub cohesion_scale: f32,
}
const _: () = assert!(
    std::mem::size_of::<SimParams>() == 32,
    "size of SimParams does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, num_particles) == 0,
    "offset of SimParams.num_particles does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, delta_time) == 4,
    "offset of SimParams.delta_time does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, separation_distance) == 8,
    "offset of SimParams.separation_distance does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, alignment_distance) == 12,
    "offset of SimParams.alignment_distance does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, cohesion_distance) == 16,
    "offset of SimParams.cohesion_distance does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, separation_scale) == 20,
    "offset of SimParams.separation_scale does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, alignment_scale) == 24,
    "offset of SimParams.alignment_scale does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, cohesion_scale) == 28,
    "offset of SimParams.cohesion_scale does not match WGSL"
);
pub const PI: f32 = 3.1415927f32;
pub const TAU: f32 = 6.2831855f32;
pub mod params {
    pub const GROUP: u32 = 0u32;
    pub const BINDING: u32 = 0u32;
    pub const LAYOUT: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: BINDING,
        visibility: wgpu::ShaderStages::all(),
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    pub type Resource<'a> = wgpu::BufferBinding<'a>;
    pub fn bind_group_entry(resource: Resource) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: BINDING,
            resource: wgpu::BindingResource::Buffer(resource),
        }
    }
}
pub mod particles_src {
    pub const GROUP: u32 = 0u32;
    pub const BINDING: u32 = 1u32;
    pub const LAYOUT: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: BINDING,
        visibility: wgpu::ShaderStages::all(),
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    pub type Resource<'a> = wgpu::BufferBinding<'a>;
    pub fn bind_group_entry(resource: Resource) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: BINDING,
            resource: wgpu::BindingResource::Buffer(resource),
        }
    }
}
pub mod particles_dst {
    pub const GROUP: u32 = 0u32;
    pub const BINDING: u32 = 2u32;
    pub const LAYOUT: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: BINDING,
        visibility: wgpu::ShaderStages::all(),
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    pub type Resource<'a> = wgpu::BufferBinding<'a>;
    pub fn bind_group_entry(resource: Resource) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: BINDING,
            resource: wgpu::BindingResource::Buffer(resource),
        }
    }
}
impl Particle {
    pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: std::mem::offset_of!(Particle, pos) as u64,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: std::mem::offset_of!(Particle, vel) as u64,
            shader_location: 1,
        },
    ];
    pub const fn vertex_buffer_layout(
        step_mode: wgpu::VertexStepMode,
    ) -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Particle>() as u64,
            step_mode,
            attributes: &Particle::VERTEX_ATTRIBUTES,
        }
    }
}
pub const BOIDS_CS_WORKGROUP_SIZE: [u32; 3] = [256, 1, 1];
pub const ENTRY_BOIDS_VS: &str = "boids_vs";
pub const ENTRY_BOIDS_FS: &str = "boids_fs";
pub const ENTRY_BOIDS_CS: &str = "boids_cs";
#[derive(Debug)]
pub struct VertexEntry<const N: usize> {
    pub entry_point: &'static str,
    pub buffers: [wgpu::VertexBufferLayout<'static>; N],
    pub constants: std::collections::HashMap<String, f64>,
}
pub fn vertex_state<'a, const N: usize>(
    module: &'a wgpu::ShaderModule,
    entry: &'a VertexEntry<N>,
) -> wgpu::VertexState<'a> {
    wgpu::VertexState {
        module,
        entry_point: entry.entry_point,
        buffers: &entry.buffers,
        compilation_options: wgpu::PipelineCompilationOptions {
            constants: &entry.constants,
            ..Default::default()
        },
    }
}
pub fn boids_vs_entry(particle: wgpu::VertexStepMode) -> VertexEntry<1> {
    VertexEntry {
        entry_point: ENTRY_BOIDS_VS,
        buffers: [Particle::vertex_buffer_layout(particle)],
        constants: Default::default(),
    }
}
#[derive(Debug)]
pub struct FragmentEntry<const N: usize> {
    pub entry_point: &'static str,
    pub targets: [Option<wgpu::ColorTargetState>; N],
    pub constants: std::collections::HashMap<String, f64>,
}
pub fn fragment_state<'a, const N: usize>(
    module: &'a wgpu::ShaderModule,
    entry: &'a FragmentEntry<N>,
) -> wgpu::FragmentState<'a> {
    wgpu::FragmentState {
        module,
        entry_point: entry.entry_point,
        targets: &entry.targets,
        compilation_options: wgpu::PipelineCompilationOptions {
            constants: &entry.constants,
            ..Default::default()
        },
    }
}
pub fn boids_fs_entry(targets: [Option<wgpu::ColorTargetState>; 1]) -> FragmentEntry<1> {
    FragmentEntry {
        entry_point: ENTRY_BOIDS_FS,
        targets,
        constants: Default::default(),
    }
}
pub fn create_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
    let source = std :: borrow :: Cow :: Borrowed ("const PI: f32 = 3.14159265358979323846264338327950288;\r\nconst TAU: f32 = 6.28318530717958647692528676655900577;\r\n\r\nstruct Particle {\r\n    @location(0) pos: vec2<f32>,\r\n    @location(1) vel: vec2<f32>,\r\n};\r\n\r\nstruct SimParams {\r\n    num_particles: u32,\r\n    delta_time: f32,\r\n    separation_distance: f32,\r\n    alignment_distance: f32,\r\n    cohesion_distance: f32,\r\n    separation_scale: f32,\r\n    alignment_scale: f32,\r\n    cohesion_scale: f32,\r\n};\r\n\r\nstruct VertexOutput {\r\n  @builtin(position) position: vec4f,\r\n  @location(0) color: vec4f,\r\n}\r\n\r\nvar<private> VERTEX_POSITIONS: array<vec2f, 3> = array(vec2f(-0.01, -0.02), vec2f(0.01, -0.02), vec2f(0.00, 0.02));\r\n\r\n@vertex\r\nfn boids_vs(\r\n    particle: Particle,\r\n    @builtin(vertex_index) vertex_index: u32,\r\n) -> VertexOutput {\r\n    let position = 0.2 * VERTEX_POSITIONS[vertex_index];\r\n    let angle = -atan2(particle.vel.x, particle.vel.y);\r\n    let pos = vec2<f32>(\r\n        position.x * cos(angle) - position.y * sin(angle),\r\n        position.x * sin(angle) + position.y * cos(angle)\r\n    );\r\n\r\n    var output: VertexOutput;\r\n    output.position = vec4(pos + particle.pos, 0., 1.);\r\n    output.color = vec4f(\r\n        saturate(2. * cos(angle)),\r\n        saturate(2. * cos(angle - (TAU / 3.))),\r\n        saturate(2. * cos(angle - (2. * TAU / 3.))),\r\n        1.\r\n    );\r\n    return output;\r\n}\r\n\r\n@fragment\r\nfn boids_fs(@location(0) color: vec4f) -> @location(0) vec4f {\r\n    return color;\r\n}\r\n\r\n@group(0) @binding(0) var<uniform> params : SimParams;\r\n@group(0) @binding(1) var<storage, read> particles_src : array<Particle>;\r\n@group(0) @binding(2) var<storage, read_write> particles_dst : array<Particle>;\r\n\r\n// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp\r\n@compute @workgroup_size(256)\r\nfn boids_cs(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {\r\n    let index = global_invocation_id.x;\r\n    if index >= params.num_particles {\r\n        return;\r\n    }\r\n\r\n    let me = particles_src[index];\r\n\r\n    var separation_vel = vec2f(0.);\r\n    var alignment_vel = vec2f(0.);\r\n    var alignment_count = 0u;\r\n    var center_of_mass = vec2f(0.);\r\n    var cohesion_count = 0u;\r\n\r\n    for (var i = 0u; i < params.num_particles; i++) {\r\n        if i == index {\r\n            continue;\r\n        }\r\n\r\n        let other = particles_src[i];\r\n\r\n        if distance(me.pos, other.pos) < params.separation_distance {\r\n            separation_vel += me.pos - other.pos;\r\n        }\r\n        if distance(me.pos, other.pos) < params.alignment_distance {\r\n            alignment_vel += other.vel;\r\n            alignment_count += 1u;\r\n        }\r\n        if distance(me.pos, other.pos) < params.cohesion_distance {\r\n            center_of_mass += other.pos;\r\n            cohesion_count += 1u;\r\n        }\r\n    }\r\n    if alignment_count > 0 {\r\n        alignment_vel /= f32(alignment_count);\r\n    }\r\n    var cohesion_vel = vec2f(0.);\r\n    if cohesion_count > 0 {\r\n        cohesion_vel = (center_of_mass / f32(cohesion_count)) - me.pos;\r\n    }\r\n\r\n    var new_particle = me;\r\n    new_particle.vel += separation_vel * params.separation_scale;\r\n    new_particle.vel += alignment_vel * params.alignment_scale;\r\n    new_particle.vel += cohesion_vel * params.cohesion_scale;\r\n\r\n    // clamp velocity for a more pleasing simulation\r\n    new_particle.vel = normalize(new_particle.vel) * clamp(length(new_particle.vel), 0.0, 0.1);\r\n\r\n    // kinematic update\r\n    new_particle.pos += new_particle.vel * params.delta_time;\r\n\r\n    // Wrap around boundary\r\n    new_particle.pos = 2. * fract(0.5 + 0.5 * new_particle.pos) - 1.;\r\n\r\n    // Write back\r\n    particles_dst[index] = new_particle;\r\n}\r\n") ;
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(source),
    })
}
