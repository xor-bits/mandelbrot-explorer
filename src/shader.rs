use bytemuck::{Pod, Zeroable};
use srs2dge::wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, Device, PipelineLayoutDescriptor,
    ShaderStages,
};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use srs2dge::prelude::*;

//

const SHADER: &str = include_str!("main.wgsl");

//

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Ubo {
    pub aspect: f32,
    // _pad: [f32; 3],
}

pub struct FractalShader {
    inner: Shader<DefaultVertex, DefaultIndex>,
    layout: BindGroupLayout,
    device: Arc<Device>,
}

//

impl FractalShader {
    pub fn new(target: &Target) -> Self {
        let module = ShaderModule::new_wgsl_source(target, SHADER.into())
            .unwrap_or_else(|err| panic!("Shader compilation failed: {err}"));

        let layout = Self::bind_group_layout(&target.get_device());

        Self {
            inner: Shader::builder()
                .with_vertex(&module, "vs_main")
                .with_fragment(&module, "fs_main")
                .with_format(target.get_format())
                .with_baked_layout(PipelineLayoutDescriptor {
                    label: label!(),
                    bind_group_layouts: &[&layout],
                    push_constant_ranges: &[],
                })
                .with_label(label!())
                .build(target),
            layout,

            device: target.get_device(),
        }
    }
}

impl<'a> Layout<'a> for FractalShader {
    type Bindings = &'a UniformBuffer<Ubo>;

    fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: label!(),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn bind_group(&self, bindings: Self::Bindings) -> BindGroup {
        self.device.create_bind_group(&BindGroupDescriptor {
            label: label!(),
            layout: &self.layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: bindings.inner().as_entire_binding(),
            }],
        })
    }
}

impl Deref for FractalShader {
    type Target = Shader<DefaultVertex, DefaultIndex>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for FractalShader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
