struct Camera {
    origin: vec2f,
    scale: vec2f,
}

@group(0) @binding(0) var res_cur_tex: texture_storage_2d<r32uint, read>;
@group(0) @binding(1) var res_next_tex: texture_storage_2d<r32uint, write>;

@group(1) @binding(0) var<uniform> res_camera: Camera;

var<private> POSITIONS: array<vec2f, 4> = array<vec2f, 4>(
    vec2f(-0.5, -0.5),
    vec2f(00.5, -0.5),
    vec2f(-0.5, 00.5),
    vec2f(00.5, 00.5),
);

var<private> TEX_COORDS: array<vec2f, 4> = array<vec2f, 4>(
    vec2f(0., 1.),
    vec2f(1., 1.),
    vec2f(0., 0.),
    vec2f(1., 0.),
);

struct TexturedQuadVsToFs {
    @builtin(position) position: vec4f,
    @location(0) tex_coord: vec2f,
};

@vertex
fn vs_textured_quad(@builtin(vertex_index) vertex_index: u32) -> TexturedQuadVsToFs {
    let position = res_camera.scale * POSITIONS[vertex_index] + res_camera.origin;

    var result: TexturedQuadVsToFs;
    result.position = vec4f(position, 0., 1.);
    result.tex_coord = TEX_COORDS[vertex_index];
    return result;
}

@fragment
fn fs_textured_quad(vs_to_fs: TexturedQuadVsToFs) -> @location(0) vec4f {
    let dim = vec2f(textureDimensions(res_cur_tex));
    let texel_coords = vec2u(vs_to_fs.tex_coord * dim);
    let shade = f32(textureLoad(res_cur_tex, texel_coords).r);
    return vec4f(shade, shade, shade, 1.);
}

@compute @workgroup_size(8, 8)
fn paint_pixel(@builtin(global_invocation_id) coord: vec3u) {
    if coord.x == 0 && coord.y == 0 {
        textureStore(res_next_tex, coord.xy, vec4u(1, 0, 0, 0));
    } else {
        let dim = textureDimensions(res_cur_tex);
        var new_texel = 0u;
        let idim = vec2i(dim);
        let icoord = vec2i(coord.xy) + idim;
        new_texel ^= textureLoad(res_cur_tex, (icoord + vec2i(-1, -1)) % idim).r;
        new_texel ^= textureLoad(res_cur_tex, (icoord + vec2i(0, -1)) % idim).r;
        new_texel ^= textureLoad(res_cur_tex, (icoord + vec2i(1, -1)) % idim).r;
        textureStore(res_next_tex, coord.xy, vec4u(new_texel, 0, 0, 0));
    }
}