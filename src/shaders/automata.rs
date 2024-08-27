#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck :: Pod, bytemuck :: Zeroable)]
pub struct Camera {
    pub origin: [f32; 2],
    pub scale: [f32; 2],
}
const _: () = assert!(
    std::mem::size_of::<Camera>() == 16,
    "size of Camera does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(Camera, origin) == 0,
    "offset of Camera.origin does not match WGSL"
);
const _: () = assert!(
    std::mem::offset_of!(Camera, scale) == 8,
    "offset of Camera.scale does not match WGSL"
);
pub mod res_cur_tex {
    pub const GROUP: u32 = 0u32;
    pub const BINDING: u32 = 0u32;
    pub const LAYOUT: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: BINDING,
        visibility: wgpu::ShaderStages::all(),
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::ReadOnly,
            format: wgpu::TextureFormat::R32Uint,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    };
    pub type Resource<'a> = &'a wgpu::TextureView;
    pub fn bind_group_entry(resource: Resource) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: BINDING,
            resource: wgpu::BindingResource::TextureView(resource),
        }
    }
}
pub mod res_next_tex {
    pub const GROUP: u32 = 0u32;
    pub const BINDING: u32 = 1u32;
    pub const LAYOUT: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: BINDING,
        visibility: wgpu::ShaderStages::all(),
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::R32Uint,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    };
    pub type Resource<'a> = &'a wgpu::TextureView;
    pub fn bind_group_entry(resource: Resource) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: BINDING,
            resource: wgpu::BindingResource::TextureView(resource),
        }
    }
}
pub mod res_camera {
    pub const GROUP: u32 = 1u32;
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
pub const PAINT_PIXEL_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
pub const ENTRY_VS_TEXTURED_QUAD: &str = "vs_textured_quad";
pub const ENTRY_FS_TEXTURED_QUAD: &str = "fs_textured_quad";
pub const ENTRY_PAINT_PIXEL: &str = "paint_pixel";
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
pub fn vs_textured_quad_entry() -> VertexEntry<0> {
    VertexEntry {
        entry_point: ENTRY_VS_TEXTURED_QUAD,
        buffers: [],
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
pub fn fs_textured_quad_entry(targets: [Option<wgpu::ColorTargetState>; 1]) -> FragmentEntry<1> {
    FragmentEntry {
        entry_point: ENTRY_FS_TEXTURED_QUAD,
        targets,
        constants: Default::default(),
    }
}
pub fn create_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
    let source = std :: borrow :: Cow :: Borrowed ("struct Camera {\r\n    origin: vec2f,\r\n    scale: vec2f,\r\n}\r\n\r\n@group(0) @binding(0) var res_cur_tex: texture_storage_2d<r32uint, read>;\r\n@group(0) @binding(1) var res_next_tex: texture_storage_2d<r32uint, write>;\r\n\r\n@group(1) @binding(0) var<uniform> res_camera: Camera;\r\n\r\nvar<private> POSITIONS: array<vec2f, 4> = array<vec2f, 4>(\r\n    vec2f(-0.5, -0.5),\r\n    vec2f(00.5, -0.5),\r\n    vec2f(-0.5, 00.5),\r\n    vec2f(00.5, 00.5),\r\n);\r\n\r\nvar<private> TEX_COORDS: array<vec2f, 4> = array<vec2f, 4>(\r\n    vec2f(0., 1.),\r\n    vec2f(1., 1.),\r\n    vec2f(0., 0.),\r\n    vec2f(1., 0.),\r\n);\r\n\r\nstruct TexturedQuadVsToFs {\r\n    @builtin(position) position: vec4f,\r\n    @location(0) tex_coord: vec2f,\r\n};\r\n\r\n@vertex\r\nfn vs_textured_quad(@builtin(vertex_index) vertex_index: u32) -> TexturedQuadVsToFs {\r\n    let position = res_camera.scale * POSITIONS[vertex_index] + res_camera.origin;\r\n\r\n    var result: TexturedQuadVsToFs;\r\n    result.position = vec4f(position, 0., 1.);\r\n    result.tex_coord = TEX_COORDS[vertex_index];\r\n    return result;\r\n}\r\n\r\n@fragment\r\nfn fs_textured_quad(vs_to_fs: TexturedQuadVsToFs) -> @location(0) vec4f {\r\n    let dim = vec2f(textureDimensions(res_cur_tex));\r\n    let texel_coords = vec2u(vs_to_fs.tex_coord * dim);\r\n    let shade = f32(textureLoad(res_cur_tex, texel_coords).r);\r\n    return vec4f(shade, shade, shade, 1.);\r\n}\r\n\r\n@compute @workgroup_size(8, 8)\r\nfn paint_pixel(@builtin(global_invocation_id) coord: vec3u) {\r\n    if coord.x == 0 && coord.y == 0 {\r\n        textureStore(res_next_tex, coord.xy, vec4u(1, 0, 0, 0));\r\n    } else {\r\n        let dim = textureDimensions(res_cur_tex);\r\n        var new_texel = 0u;\r\n        let idim = vec2i(dim);\r\n        let icoord = vec2i(coord.xy) + idim;\r\n        new_texel ^= textureLoad(res_cur_tex, (icoord + vec2i(-1, -1)) % idim).r;\r\n        new_texel ^= textureLoad(res_cur_tex, (icoord + vec2i(0, -1)) % idim).r;\r\n        new_texel ^= textureLoad(res_cur_tex, (icoord + vec2i(1, -1)) % idim).r;\r\n        textureStore(res_next_tex, coord.xy, vec4u(new_texel, 0, 0, 0));\r\n    }\r\n}") ;
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(source),
    })
}
