//! This module contains rust types and descriptors derived from shader.wgsl. I could use wgsl_to_wgpu or wgsl_bindgen
//! to generate this, and I may try to do so in the future. But those crates are missing some features and that I'd like.
//! Neither crate handles having multiple vertex/fragment entry points well, nor reusing bind group indices.

use std::{borrow::Cow, mem};

use bytemuck::{Pod, Zeroable};
use eframe::wgpu;
use glam::Mat4;

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct Camera {
    // from camera to screen
    pub proj: Mat4,
    // from screen to camera
    pub proj_inv: Mat4,
    // from world to camera
    pub view: Mat4,
    // from world to camera
    pub view_inv: Mat4,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub _pos: [f32; 4],
    pub _tex_coord: [f32; 2],
}

pub const SHADER_MODULE_DESC: wgpu::ShaderModuleDescriptor<'static> =
    wgpu::ShaderModuleDescriptor {
        label: Some("shader.wgsl"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    };

pub const BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(mem::size_of::<Camera>() as _),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    };

pub mod vs_mesh {
    use super::*;

    pub const ENTRY_NAME: &str = "vs_mesh";

    pub const BUFFER_LAYOUTS: [wgpu::VertexBufferLayout<'static>; 1] = [wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
    }];
}

pub mod fs_mesh {
    pub const ENTRY_NAME: &str = "fs_mesh";
}

pub mod fs_wireframe {
    pub const ENTRY_NAME: &str = "fs_wireframe";
}
