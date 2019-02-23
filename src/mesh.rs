use crate::vulkan::{Buffer, Texture};

pub struct Mesh {
    vertices: Buffer,
    indices: Buffer,
    index_count: usize,
    texture: Texture,
}

impl Mesh {
    pub fn new(vertices: Buffer, indices: Buffer, index_count: usize, texture: Texture) -> Self {
        Self {
            vertices,
            indices,
            index_count,
            texture,
        }
    }
}

impl Mesh {
    pub fn vertices(&self) -> &Buffer {
        &self.vertices
    }

    pub fn indices(&self) -> &Buffer {
        &self.indices
    }

    pub fn index_count(&self) -> usize {
        self.index_count
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}
