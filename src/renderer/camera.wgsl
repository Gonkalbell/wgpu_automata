#define_import_path camera

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
