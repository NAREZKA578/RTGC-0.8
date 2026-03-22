//! Asset loader - Universal asset loading system

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Handle to a loaded asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHandle(u64);

impl AssetHandle {
    pub const fn null() -> Self {
        Self(0)
    }
    
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }
}

/// Supported asset types
#[derive(Debug, Clone)]
pub enum AssetType {
    Texture,
    Mesh,
    Shader,
    Audio,
    Font,
    Config,
    Model,
}

/// Loaded asset data
#[derive(Debug, Clone)]
pub enum AssetData {
    Texture {
        width: u32,
        height: u32,
        channels: u8,
        data: Vec<u8>,
    },
    Mesh {
        vertices: Vec<f32>,
        indices: Vec<u32>,
    },
    Shader {
        source: String,
        shader_type: ShaderStage,
    },
    Audio {
        sample_rate: u32,
        channels: u16,
        samples: Vec<f32>,
    },
    Font {
        name: String,
        size: u32,
        data: Vec<u8>,
    },
    Config {
        content: String,
    },
    Model {
        path: PathBuf,
        data: Vec<u8>,
    },
}

/// Shader stage type
#[derive(Debug, Clone, Copy)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
}

/// Metadata for an asset
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    pub name: String,
    pub asset_type: AssetType,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub load_time_ms: f64,
}

/// Asset loading errors
#[derive(Debug)]
pub enum AssetLoadError {
    IoError(std::io::Error),
    InvalidFormat(String),
    NotFound(String),
    UnsupportedType(String),
    DecodeError(String),
    UnsupportedFormat(String),
}

impl From<std::io::Error> for AssetLoadError {
    fn from(err: std::io::Error) -> Self {
        AssetLoadError::IoError(err)
    }
}

impl std::fmt::Display for AssetLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetLoadError::IoError(e) => write!(f, "IO error: {}", e),
            AssetLoadError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            AssetLoadError::NotFound(path) => write!(f, "Asset not found: {}", path),
            AssetLoadError::UnsupportedType(ty) => write!(f, "Unsupported type: {}", ty),
            AssetLoadError::DecodeError(msg) => write!(f, "Decode error: {}", msg),
            AssetLoadError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
        }
    }
}

impl std::error::Error for AssetLoadError {}

/// Asset loader configuration
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    pub root_path: PathBuf,
    pub cache_size_mb: usize,
    pub async_loading: bool,
    pub hot_reload: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("assets"),
            cache_size_mb: 512,
            async_loading: true,
            hot_reload: false,
        }
    }
}

/// Universal asset loader
pub struct AssetLoader {
    config: LoaderConfig,
    assets: HashMap<AssetHandle, Arc<AssetData>>,
    metadata: HashMap<AssetHandle, AssetMetadata>,
    next_handle: u64,
    path_to_handle: HashMap<PathBuf, AssetHandle>,
}

impl AssetLoader {
    /// Creates a new asset loader with default config
    pub fn new() -> Self {
        Self::with_config(LoaderConfig::default())
    }

    /// Creates a new asset loader with custom config
    pub fn with_config(config: LoaderConfig) -> Self {
        Self {
            config,
            assets: HashMap::new(),
            metadata: HashMap::new(),
            next_handle: 1,
            path_to_handle: HashMap::new(),
        }
    }

    /// Generates a new unique asset handle
    fn generate_handle(&mut self) -> AssetHandle {
        let handle = AssetHandle(self.next_handle);
        self.next_handle += 1;
        handle
    }

    /// Loads an asset from a file path
    pub fn load<P: AsRef<Path>>(&mut self, path: P, asset_type: AssetType) -> Result<AssetHandle, AssetLoadError> {
        let path = path.as_ref().to_path_buf();
        
        // Check if already loaded
        if let Some(&handle) = self.path_to_handle.get(&path) {
            return Ok(handle);
        }

        let full_path = self.config.root_path.join(&path);
        
        if !full_path.exists() {
            return Err(AssetLoadError::NotFound(full_path.display().to_string()));
        }

        let start_time = std::time::Instant::now();
        
        let data = match asset_type {
            AssetType::Texture => self.load_texture(&full_path)?,
            AssetType::Mesh => self.load_mesh(&full_path)?,
            AssetType::Shader => self.load_shader(&full_path)?,
            AssetType::Audio => self.load_audio(&full_path)?,
            AssetType::Font => self.load_font(&full_path)?,
            AssetType::Config => self.load_config(&full_path)?,
            AssetType::Model => self.load_model(&full_path)?,
        };

        let load_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let size_bytes = full_path.metadata()?.len();

        let handle = self.generate_handle();
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let metadata = AssetMetadata {
            name: name.clone(),
            asset_type: asset_type.clone(),
            path: path.clone(),
            size_bytes,
            load_time_ms,
        };

        let arc_data = Arc::new(data);
        self.assets.insert(handle, arc_data);
        self.metadata.insert(handle, metadata);
        self.path_to_handle.insert(path, handle);

        Ok(handle)
    }

    /// Gets a reference to a loaded asset
    pub fn get(&self, handle: AssetHandle) -> Option<Arc<AssetData>> {
        self.assets.get(&handle).cloned()
    }

    /// Gets metadata for a loaded asset
    pub fn get_metadata(&self, handle: AssetHandle) -> Option<&AssetMetadata> {
        self.metadata.get(&handle)
    }

    /// Unloads an asset
    pub fn unload(&mut self, handle: AssetHandle) -> bool {
        if let Some(metadata) = self.metadata.remove(&handle) {
            self.path_to_handle.remove(&metadata.path);
            self.assets.remove(&handle);
            true
        } else {
            false
        }
    }

    /// Unloads all assets
    pub fn unload_all(&mut self) {
        self.assets.clear();
        self.metadata.clear();
        self.path_to_handle.clear();
    }

    /// Returns the number of loaded assets
    pub fn loaded_count(&self) -> usize {
        self.assets.len()
    }

    /// Returns total memory usage in bytes
    pub fn memory_usage(&self) -> u64 {
        self.metadata.values().map(|m| m.size_bytes).sum()
    }

    /// Loads a texture (PNG, JPG, etc.)
    pub fn load_texture(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        // Placeholder - would use image crate in production
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // For now, return placeholder texture data
        Ok(AssetData::Texture {
            width: 256,
            height: 256,
            channels: 4,
            data: vec![255u8; 256 * 256 * 4],
        })
    }

    /// Loads a mesh (OBJ, FBX, glTF, etc.)
    fn load_mesh(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        // Placeholder - would use a mesh loading library
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "obj" => self.load_obj(path),
            _ => Err(AssetLoadError::UnsupportedFormat(format!("Unknown mesh format: {}", extension))),
        }
    }

    /// Loads an OBJ mesh file
    fn load_obj(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Simple OBJ parser placeholder
        for line in std::io::BufRead::lines(reader) {
            let line = line?;
            if line.starts_with("v ") {
                // Parse vertex position
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        vertices.extend_from_slice(&[x, y, z]);
                    }
                }
            } else if line.starts_with("f ") {
                // Parse face (simple triangulation)
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    for i in 1..parts.len() - 2 {
                        if let (Some(v1), Some(v2), Some(v3)) = (
                            parts[i].split('/').next().and_then(|s| s.parse::<u32>().ok()),
                            parts[i + 1].split('/').next().and_then(|s| s.parse::<u32>().ok()),
                            parts[i + 2].split('/').next().and_then(|s| s.parse::<u32>().ok()),
                        ) {
                            indices.extend_from_slice(&[v1 - 1, v2 - 1, v3 - 1]);
                        }
                    }
                }
            }
        }

        Ok(AssetData::Mesh { vertices, indices })
    }

    /// Loads a shader file
    fn load_shader(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        let mut file = File::open(path)?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;

        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let shader_type = match extension.to_lowercase().as_str() {
            "vert" | "glslv" => ShaderStage::Vertex,
            "frag" | "glslf" => ShaderStage::Fragment,
            "comp" | "glslc" => ShaderStage::Compute,
            "geom" | "glslg" => ShaderStage::Geometry,
            "tesc" => ShaderStage::TessellationControl,
            "tese" => ShaderStage::TessellationEvaluation,
            _ => return Err(AssetLoadError::UnsupportedType(format!("Unknown shader extension: {}", extension))),
        };

        Ok(AssetData::Shader { source, shader_type })
    }

    /// Loads an audio file (WAV, OGG, etc.)
    fn load_audio(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        // Placeholder - would use hound or ogg crate
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "wav" => self.load_wav(path),
            _ => Err(AssetLoadError::UnsupportedType(format!("Unknown audio format: {}", extension))),
        }
    }

    /// Loads a WAV audio file
    fn load_wav(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        // Placeholder implementation
        Ok(AssetData::Audio {
            sample_rate: 44100,
            channels: 2,
            samples: vec![0.0f32; 44100],
        })
    }

    /// Loads a font file
    fn load_font(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("font")
            .to_string();

        Ok(AssetData::Font {
            name,
            size: 16,
            data,
        })
    }

    /// Loads a config file (JSON, TOML, etc.)
    fn load_config(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(AssetData::Config { content })
    }

    /// Loads a 3D model file (glTF/GLB)
    fn load_model(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "gltf" | "glb" => self.load_gltf(path),
            _ => Err(AssetLoadError::UnsupportedType(format!("Unknown model format: {}", extension))),
        }
    }

    /// Loads a glTF/GLB model file
    fn load_gltf(&self, path: &Path) -> Result<AssetData, AssetLoadError> {
        use gltf::{Gltf, buffer::Data};
        
        // Read file
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut data)?;
        
        // Parse glTF
        let (document, buffers, _images) = gltf::import(path)
            .map_err(|e| AssetLoadError::DecodeError(format!("Failed to import glTF: {}", e)))?;
        
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        
        // Iterate through all meshes
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buf| Some(&buffers[buf.index()]));
                
                // Read positions
                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .ok_or_else(|| AssetLoadError::InvalidFormat("No positions in mesh".to_string()))?
                    .collect();
                
                // Read normals (or generate defaults)
                let normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .map(|n| n.collect())
                    .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
                
                // Read UVs (or default to 0,0)
                let uvs: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|t| t.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
                
                // Read indices
                let indices: Vec<u32> = reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_else(|| (0..positions.len() as u32).collect());
                
                // Build interleaved vertex buffer: pos(3) + normal(3) + uv(2) = 8 floats per vertex
                let vertices: Vec<f32> = positions.iter()
                    .zip(normals.iter())
                    .zip(uvs.iter())
                    .flat_map(|((p, n), uv)| {
                        vec![p[0], p[1], p[2], n[0], n[1], n[2], uv[0], uv[1]]
                    })
                    .collect();
                
                all_vertices.extend(vertices);
                all_indices.extend(indices);
            }
        }
        
        Ok(AssetData::Mesh { 
            vertices: all_vertices, 
            indices: all_indices 
        })
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_handle() {
        let handle = AssetHandle::null();
        assert!(handle.is_null());
        
        let handle2 = AssetHandle(1);
        assert!(!handle2.is_null());
    }

    #[test]
    fn test_loader_creation() {
        let loader = AssetLoader::new();
        assert_eq!(loader.loaded_count(), 0);
        assert_eq!(loader.memory_usage(), 0);
    }

    #[test]
    fn test_loader_config() {
        let config = LoaderConfig::default();
        assert_eq!(config.cache_size_mb, 512);
        assert!(config.async_loading);
    }
}
