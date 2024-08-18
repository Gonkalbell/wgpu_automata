struct Camera {
    origin: vec2<f32>,
    scale: vec2<f32>,
}

@group(0)
@binding(0)
var<uniform> res_camera: Camera;
// @group(0)
// @binding(0)
// var res_texture: texture_2d<f32>;
// @group(0)
// @binding(1)
// var res_sampler: sampler;

var<private> POSITIONS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-0.5, -0.5),
    vec2<f32>(00.5, -0.5),
    vec2<f32>(-0.5, 00.5),
    vec2<f32>(00.5, 00.5),
);

var<private> TEX_COORDS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(0., 0.),
    vec2<f32>(1., 0.),
    vec2<f32>(0., 1.),
    vec2<f32>(1., 1.),
);

struct TexturedQuadVsToFs {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_textured_quad(@builtin(vertex_index) vertex_index: u32) -> TexturedQuadVsToFs {
    var position = res_camera.scale * POSITIONS[vertex_index] + res_camera.origin;

    var result: TexturedQuadVsToFs;
    result.position = vec4<f32>(position, 0., 1.);
    result.tex_coord = TEX_COORDS[vertex_index];
    return result;
}

@fragment
fn fs_textured_quad(vs_to_fs: TexturedQuadVsToFs) -> @location(0) vec4<f32> {
    return vec4<f32>(vs_to_fs.tex_coord, 0., 1.);
    // return textureSample(res_texture, res_sampler, vertex.tex_coord);
}
