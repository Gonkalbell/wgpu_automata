//! This module contains rust types and descriptors derived from shader.wgsl. I could use wgsl_to_wgpu or wgsl_bindgen
//! to generate this, and I may try to do so in the future. But those crates are missing some features and that I'd like.
//! Neither crate handles having multiple vertex/fragment entry points well, nor reusing bind group indices.

use std::{borrow::Cow, mem};

use bytemuck::{Pod, Zeroable};
use eframe::wgpu;
use glam::Mat4;

pub const CAMERA_GROUP: u32 = 0;
pub const SKYBOX_GROUP: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct Camera {
    // from world space to view space
    pub view: Mat4,
    // from vew space to world space
    pub view_inv: Mat4,
    // from view space to clip space
    pub proj: Mat4,
    // from clip space to view space
    pub proj_inv: Mat4,
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

pub mod vs_skybox {
    pub const ENTRY_NAME: &str = "vs_skybox";
}

pub mod fs_skybox {
    pub const ENTRY_NAME: &str = "fs_skybox";
}
