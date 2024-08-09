// Bind group indices
const CAMERA_GROUP = 0;
const MATERIAL_GROUP = 1;
const SKYBOX_GROUP = 1;

struct Camera {
    // from world space to view space
    view: mat4x4<f32>,
    // from vew space to world space
    view_inv: mat4x4<f32>,
    // from view space to clip space
    proj: mat4x4<f32>,
    // from clip space to view space
    proj_inv: mat4x4<f32>,
};

@group(CAMERA_GROUP)
@binding(0)
var<uniform> res_camera: Camera;

@group(MATERIAL_GROUP)
@binding(0)
var res_color: texture_2d<u32>;

struct MeshInterp {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_mesh(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> MeshInterp {
    var result: MeshInterp;
    result.tex_coord = tex_coord;
    result.position = res_camera.proj * res_camera.view * position;
    return result;
}

@fragment
fn fs_mesh(vertex: MeshInterp) -> @location(0) vec4<f32> {
    let tex = textureLoad(res_color, vec2<i32>(vertex.tex_coord * 256.0), 0);
    let v = f32(tex.x) / 255.0;
    return vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);
}

@fragment
fn fs_wireframe(vertex: MeshInterp) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}

struct SkyboxInterp {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec3<f32>,
};

@vertex
fn vs_skybox(@builtin(vertex_index) vertex_index: u32) -> SkyboxInterp {
    // hacky way to draw a large triangle
    let tmp1 = i32(vertex_index) / 2;
    let tmp2 = i32(vertex_index) & 1;
    let pos = vec4<f32>(
        f32(tmp1) * 4.0 - 1.0,
        f32(tmp2) * 4.0 - 1.0,
        1.0,
        1.0
    );

    var result: SkyboxInterp;
    result.position = pos;
    let dir = vec4<f32>((res_camera.proj_inv * pos).xyz, 0.0);
    result.tex_coord = (res_camera.view_inv * dir).xyz;
    return result;
}

@group(SKYBOX_GROUP)
@binding(0)
var res_texture: texture_cube<f32>;
@group(SKYBOX_GROUP)
@binding(1)
var res_sampler: sampler;

@fragment
fn fs_skybox(vertex: SkyboxInterp) -> @location(0) vec4<f32> {
    return textureSample(res_texture, res_sampler, vertex.tex_coord);
}