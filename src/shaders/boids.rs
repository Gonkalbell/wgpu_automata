#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck :: Pod, bytemuck :: Zeroable)]
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
#[derive(Debug, Copy, Clone, PartialEq, bytemuck :: Pod, bytemuck :: Zeroable)]
pub struct SimParams {
    pub deltaT: f32,
    pub rule1Distance: f32,
    pub rule2Distance: f32,
    pub rule3Distance: f32,
    pub rule1Scale: f32,
    pub rule2Scale: f32,
    pub rule3Scale: f32,
}
const _: () = assert!(
    std::mem::size_of::<SimParams>() == 28,
    "size of SimParams does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, deltaT) == 0,
    "offset of SimParams.deltaT does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, rule1Distance) == 4,
    "offset of SimParams.rule1Distance does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, rule2Distance) == 8,
    "offset of SimParams.rule2Distance does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, rule3Distance) == 12,
    "offset of SimParams.rule3Distance does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, rule1Scale) == 16,
    "offset of SimParams.rule1Scale does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, rule2Scale) == 20,
    "offset of SimParams.rule2Scale does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(SimParams, rule3Scale) == 24,
    "offset of SimParams.rule3Scale does not match WGSL"
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
pub mod particlesSrc {
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
pub mod particlesDst {
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
pub const MAIN_WORKGROUP_SIZE: [u32; 3] = [64, 1, 1];
pub const ENTRY_MAIN_VS: &str = "main_vs";
pub const ENTRY_MAIN_FS: &str = "main_fs";
pub const ENTRY_MAIN: &str = "main";
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
pub fn main_vs_entry(particle: wgpu::VertexStepMode) -> VertexEntry<1> {
    VertexEntry {
        entry_point: ENTRY_MAIN_VS,
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
pub fn main_fs_entry(targets: [Option<wgpu::ColorTargetState>; 1]) -> FragmentEntry<1> {
    FragmentEntry {
        entry_point: ENTRY_MAIN_FS,
        targets,
        constants: Default::default(),
    }
}
pub fn create_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
    let source = std :: borrow :: Cow :: Borrowed ("const PI: f32 = 3.14159265358979323846264338327950288;\nconst TAU: f32 = 6.28318530717958647692528676655900577;\n\nstruct Particle {\n    @location(0) pos: vec2<f32>,\n    @location(1) vel: vec2<f32>,\n};\n\nstruct SimParams {\n    deltaT: f32,\n    rule1Distance: f32,\n    rule2Distance: f32,\n    rule3Distance: f32,\n    rule1Scale: f32,\n    rule2Scale: f32,\n    rule3Scale: f32,\n};\n\nstruct VertexOutput {\n  @builtin(position) position: vec4f,\n  @location(0) color: vec4f,\n}\n\nvar<private> VERTEX_POSITIONS: array<vec2f, 3> = array(vec2f(-0.01, -0.02), vec2f(0.01, -0.02), vec2f(0.00, 0.02));\n\n@vertex\nfn main_vs(\n    particle: Particle,\n    @builtin(vertex_index) vertex_index: u32,\n) -> VertexOutput {\n    let position = VERTEX_POSITIONS[vertex_index];\n    let angle = -atan2(particle.vel.x, particle.vel.y);\n    let pos = vec2<f32>(\n        position.x * cos(angle) - position.y * sin(angle),\n        position.x * sin(angle) + position.y * cos(angle)\n    );\n\n    var output: VertexOutput;\n    output.position = vec4(pos + particle.pos, 0., 1.);\n    output.color = vec4f(\n        saturate(cos(angle)),\n        saturate(cos(angle - (TAU / 3.))),\n        saturate(cos(angle - (2. * TAU / 3.))),\n        1.\n    );\n    return output;\n}\n\n@fragment\nfn main_fs(@location(0) color: vec4f) -> @location(0) vec4f {\n    return color;\n}\n\n\n@group(0) @binding(0) var<uniform> params : SimParams;\n@group(0) @binding(1) var<storage, read> particlesSrc : array<Particle>;\n@group(0) @binding(2) var<storage, read_write> particlesDst : array<Particle>;\n\n// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp\n@compute @workgroup_size(64)\nfn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {\n    let total = arrayLength(&particlesSrc);\n    let index = global_invocation_id.x;\n    if index >= total {\n        return;\n    }\n\n    var vPos: vec2<f32> = particlesSrc[index].pos;\n    var vVel: vec2<f32> = particlesSrc[index].vel;\n\n    var cMass: vec2<f32> = vec2<f32>(0.0, 0.0);\n    var cVel: vec2<f32> = vec2<f32>(0.0, 0.0);\n    var colVel: vec2<f32> = vec2<f32>(0.0, 0.0);\n    var cMassCount: i32 = 0;\n    var cVelCount: i32 = 0;\n\n    var i: u32 = 0u;\n    loop {\n        if i >= total {\n            break;\n        }\n        if i == index {\n            continue;\n        }\n\n        let pos = particlesSrc[i].pos;\n        let vel = particlesSrc[i].vel;\n\n        if distance(pos, vPos) < params.rule1Distance {\n            cMass += pos;\n            cMassCount += 1;\n        }\n        if distance(pos, vPos) < params.rule2Distance {\n            colVel -= pos - vPos;\n        }\n        if distance(pos, vPos) < params.rule3Distance {\n            cVel += vel;\n            cVelCount += 1;\n        }\n\n        continuing {\n            i = i + 1u;\n        }\n    }\n    if cMassCount > 0 {\n        cMass = cMass * (1.0 / f32(cMassCount)) - vPos;\n    }\n    if cVelCount > 0 {\n        cVel *= 1.0 / f32(cVelCount);\n    }\n\n    vVel = vVel + (cMass * params.rule1Scale) + (colVel * params.rule2Scale) + (cVel * params.rule3Scale);\n\n    // clamp velocity for a more pleasing simulation\n    vVel = normalize(vVel) * clamp(length(vVel), 0.0, 0.1);\n\n    // kinematic update\n    vPos += vVel * params.deltaT;\n\n    // Wrap around boundary\n    if vPos.x < -1.0 {\n        vPos.x = 1.0;\n    }\n    if vPos.x > 1.0 {\n        vPos.x = -1.0;\n    }\n    if vPos.y < -1.0 {\n        vPos.y = 1.0;\n    }\n    if vPos.y > 1.0 {\n        vPos.y = -1.0;\n    }\n\n    // Write back\n    particlesDst[index] = Particle(vPos, vVel);\n}\n") ;
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(source),
    })
}
