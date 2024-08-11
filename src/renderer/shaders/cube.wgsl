#import bgroup_camera::res_camera

@group(1)
@binding(0)
var res_color: texture_2d<u32>;

struct Vertex {
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
}

struct MeshInterp {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_mesh(in: Vertex) -> MeshInterp {
    var out: MeshInterp;
    out.tex_coord = in.tex_coord;
    out.position = res_camera.proj * res_camera.view * in.position;
    return out;
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