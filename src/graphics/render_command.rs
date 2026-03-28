//! Render Command - Encapsulates rendering operations for the render queue
//! DEBUG: Handle использует только Clone (без Copy) для избежания конфликтов

use crate::graphics::mesh::Mesh;
use crate::graphics::texture::Texture;
use crate::graphics::material::Material;
use crate::graphics::particles::ParticleSystem;
use nalgebra::{Matrix4, Vector3};

/// Unique handle for resources
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Handle<T>(u64, std::marker::PhantomData<T>);

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0, std::marker::PhantomData)
    }
}

impl<T> Handle<T> {
    pub fn new(id: u64) -> Self {
        Self(id, std::marker::PhantomData)
    }

    pub fn id(&self) -> u64 {
        self.0
    }

    pub fn null() -> Self {
        Self(0, std::marker::PhantomData)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

/// Render command types for the render queue
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Render a mesh with material
    Mesh {
        mesh: Handle<Mesh>,
        material: Handle<Material>,
        transform: Matrix4<f32>,
        sort_key: u64,
    },
    /// Render particle system
    ParticleSystem {
        system: Handle<ParticleSystem>,
        transform: Matrix4<f32>,
        sort_key: u64,
    },
    /// UI drawing command
    UIDraw {
        texture: Handle<Texture>,
        position: Vector3<f32>,
        size: Vector3<f32>,
        color: [f32; 4],
        sort_key: u64,
    },
    /// Debug line drawing
    DebugLine {
        start: Vector3<f32>,
        end: Vector3<f32>,
        color: [f32; 4],
        sort_key: u64,
    },
    /// Skybox rendering
    Skybox {
        texture: Handle<Texture>,
        rotation: Matrix4<f32>,
        sort_key: u64,
    },
    /// Terrain chunk rendering
    TerrainChunk {
        chunk_id: u64,
        mesh: Handle<Mesh>,
        material: Handle<Material>,
        transform: Matrix4<f32>,
        lod_level: u32,
        sort_key: u64,
    },
}

impl RenderCommand {
    /// Get the sort key for this command
    pub fn sort_key(&self) -> u64 {
        match self {
            RenderCommand::Mesh { sort_key, .. } => *sort_key,
            RenderCommand::ParticleSystem { sort_key, .. } => *sort_key,
            RenderCommand::UIDraw { sort_key, .. } => *sort_key,
            RenderCommand::DebugLine { sort_key, .. } => *sort_key,
            RenderCommand::Skybox { sort_key, .. } => *sort_key,
            RenderCommand::TerrainChunk { sort_key, .. } => *sort_key,
        }
    }

    /// Set the sort key for this command
    pub fn set_sort_key(&mut self, key: u64) {
        match self {
            RenderCommand::Mesh { sort_key, .. } => *sort_key = key,
            RenderCommand::ParticleSystem { sort_key, .. } => *sort_key = key,
            RenderCommand::UIDraw { sort_key, .. } => *sort_key = key,
            RenderCommand::DebugLine { sort_key, .. } => *sort_key = key,
            RenderCommand::Skybox { sort_key, .. } => *sort_key = key,
            RenderCommand::TerrainChunk { sort_key, .. } => *sort_key = key,
        }
    }

    /// Get the material handle if applicable
    pub fn material_handle(&self) -> Option<Handle<Material>> {
        match self {
            RenderCommand::Mesh { material, .. } => Some(material.clone()),
            RenderCommand::TerrainChunk { material, .. } => Some(material.clone()),
            _ => None,
        }
    }

    /// Get the transform matrix if applicable
    pub fn transform(&self) -> Option<&Matrix4<f32>> {
        match self {
            RenderCommand::Mesh { transform, .. } => Some(transform),
            RenderCommand::ParticleSystem { transform, .. } => Some(transform),
            RenderCommand::Skybox { rotation, .. } => Some(rotation),
            RenderCommand::TerrainChunk { transform, .. } => Some(transform),
            _ => None,
        }
    }
}

/// Builder for creating render commands
pub struct RenderCommandBuilder {
    command_type: CommandType,
}

#[derive(Debug, Clone)]
enum CommandType {
    Mesh,
    ParticleSystem,
    UIDraw,
    DebugLine,
    Skybox,
    TerrainChunk,
}

impl RenderCommandBuilder {
    pub fn mesh() -> Self {
        Self {
            command_type: CommandType::Mesh,
        }
    }

    pub fn particle_system() -> Self {
        Self {
            command_type: CommandType::ParticleSystem,
        }
    }

    pub fn ui_draw() -> Self {
        Self {
            command_type: CommandType::UIDraw,
        }
    }

    pub fn debug_line() -> Self {
        Self {
            command_type: CommandType::DebugLine,
        }
    }

    pub fn skybox() -> Self {
        Self {
            command_type: CommandType::Skybox,
        }
    }

    pub fn terrain_chunk() -> Self {
        Self {
            command_type: CommandType::TerrainChunk,
        }
    }
}
