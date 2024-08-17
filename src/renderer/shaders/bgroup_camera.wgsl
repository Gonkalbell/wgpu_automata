#define_import_path bgroup_camera

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

@group(0) @binding(0) var<uniform> res_camera: Camera;
