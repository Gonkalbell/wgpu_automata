//! This module contains rust types and descriptors derived from shader.wgsl. I could use wgsl_to_wgpu or wgsl_bindgen
//! to generate this, and I may try to do so in the future. But those crates are missing some features and that I'd like.
//! Neither crate handles having multiple vertex/fragment entry points well, nor reusing bind group indices.

use std::{borrow::Cow, mem};

use bytemuck::{Pod, Zeroable};
use eframe::wgpu;
use glam::Mat4;

pub const CAMERA_GROUP: u32 = 0;
pub const MATERIAL_GROUP: u32 = 1;
pub const SKYBOX_GROUP: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct Camera {
    // from view space to clip space
    pub proj: Mat4,
    // from clip space to view space
    pub proj_inv: Mat4,
    // from world space to view space
    pub view: Mat4,
    // from vew space to world space
    pub view_inv: Mat4,
}

pub const SHADER_MODULE_DESC: wgpu::ShaderModuleDescriptor<'static> =
    wgpu::ShaderModuleDescriptor {
        label: Some("shader.wgsl"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    };

pub const CAMERA_BGROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("camera"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(mem::size_of::<Camera>() as _),
            },
            count: None,
        }],
    };

pub const MATERIAL_BGROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("material"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                sample_type: wgpu::TextureSampleType::Uint,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }],
    };

pub const SKYBOX_BGROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("skybox"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::Cube,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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

    #[repr(C)]
    #[derive(Clone, Copy, Pod, Zeroable)]
    pub struct Vertex {
        pub pos: [f32; 4],
        pub tex_coord: [f32; 2],
    }
}

pub mod fs_mesh {
    pub const ENTRY_NAME: &str = "fs_mesh";
}

pub mod fs_wireframe {
    pub const ENTRY_NAME: &str = "fs_wireframe";
}

pub mod vs_skybox {
    pub const ENTRY_NAME: &str = "vs_skybox";
}

pub mod fs_skybox {
    pub const ENTRY_NAME: &str = "fs_skybox";
}
