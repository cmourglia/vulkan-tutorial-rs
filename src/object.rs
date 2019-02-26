use crate::{mesh::Mesh, vulkan::Buffer};
use ash::vk;
use cgmath::Matrix4;
use std::rc::Rc;

pub struct Object {
    mesh: Rc<Mesh>,
    uniforms: Vec<Buffer>,
    descriptor_sets: Vec<vk::DescriptorSet>,
    transform: Matrix4<f32>,
}

impl Object {
    pub fn new(
        mesh: Rc<Mesh>,
        uniforms: Vec<Buffer>,
        descriptor_sets: Vec<vk::DescriptorSet>,
        transform: Matrix4<f32>,
    ) -> Self {
        Self {
            mesh,
            uniforms,
            descriptor_sets,
            transform,
        }
    }
}

impl Object {
    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn uniforms(&self, index: usize) -> &Buffer {
        &self.uniforms[index]
    }

    pub fn descriptor_sets(&self, index: usize) -> vk::DescriptorSet {
        self.descriptor_sets[index]
    }

    pub fn transform(&self) -> Matrix4<f32> {
        self.transform
    }
}
