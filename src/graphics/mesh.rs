use glow::{Context, HasContext, NativeVertexArray, NativeBuffer};
use std::sync::Arc;
use nalgebra::Vector3;

/// Handle to a mesh resource
#[derive(Debug, Clone)]
pub struct MeshHandle {
    pub mesh: Arc<Mesh>,
}

impl MeshHandle {
    pub fn new(mesh: Mesh) -> Self {
        Self {
            mesh: Arc::new(mesh),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

unsafe impl Send for Mesh {}
unsafe impl Sync for Mesh {}

pub struct MeshInner {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    indices_count: i32,
}

#[derive(Clone)]
pub struct Mesh {
    inner: Arc<MeshInner>,
}

impl std::fmt::Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("indices_count", &self.inner.indices_count)
            .finish()
    }
}

impl Mesh {
    /// Create a new mesh from vertices and indices
    pub fn new(gl: &Context, vertices: &[Vertex], indices: &[u32]) -> Result<Self, String> {
        unsafe {
            let vao = gl.create_vertex_array().map_err(|e| format!("Failed to create VAO: {}", e))?;
            gl.bind_vertex_array(Some(vao));

            let vbo = gl.create_buffer().map_err(|e| format!("Failed to create VBO: {}", e))?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                glow::STATIC_DRAW,
            );

            let ebo = gl.create_buffer().map_err(|e| format!("Failed to create EBO: {}", e))?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STATIC_DRAW,
            );

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 32, 0);

            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 32, 12);

            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 32, 24);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Ok(Mesh {
                inner: Arc::new(MeshInner {
                    vao,
                    vbo,
                    ebo,
                    indices_count: indices.len() as i32,
                }),
            })
        }
    }

    /// Create mesh from raw vertex data with normals
    pub fn new_with_normals(gl: &Context, vertices: &[f32], indices: &[u32]) -> Result<Self, String> {
        // vertices should be interleaved: pos_x, pos_y, pos_z, norm_x, norm_y, norm_z, tex_u, tex_v
        // Convert to Vertex structs
        let vertex_count = vertices.len() / 8;
        let mut vertex_data = Vec::with_capacity(vertex_count);
        for i in 0..vertex_count {
            let base = i * 8;
            vertex_data.push(Vertex {
                position: [vertices[base], vertices[base + 1], vertices[base + 2]],
                normal: [vertices[base + 3], vertices[base + 4], vertices[base + 5]],
                tex_coords: [vertices[base + 6], vertices[base + 7]],
            });
        }
        Self::new(gl, &vertex_data, indices)
    }
    
    /// Create a placeholder mesh (for async loading)
    pub fn new_placeholder() -> Self {
        use std::num::NonZero;
        // Используем unsafe new_unchecked с валидным non-zero значением
        // Placeholder mesh используется как временная заглушка до загрузки реальной модели
        Self {
            inner: Arc::new(MeshInner {
                vao: NativeVertexArray(NonZero::new_unchecked(1)),
                vbo: NativeBuffer(NonZero::new_unchecked(1)),
                ebo: NativeBuffer(NonZero::new_unchecked(1)),
                indices_count: 0,
            }),
        }
    }
    
    /// Create an empty mesh (for error cases)
    pub fn empty(gl: &Context) -> Self {
        Self::new_placeholder()
    }

    /// Generate a hash key for vertex/indice data for caching
    pub fn generate_mesh_key(vertices: &[f32], indices: &[u32]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        vertices.len().hash(&mut hasher);
        indices.len().hash(&mut hasher);
        // Hash first and last few elements as a quick fingerprint
        if vertices.len() >= 8 {
            vertices[0].to_bits().hash(&mut hasher);
            vertices[vertices.len() - 1].to_bits().hash(&mut hasher);
        }
        if indices.len() >= 2 {
            indices[0].hash(&mut hasher);
            indices[indices.len() - 1].hash(&mut hasher);
        }
        hasher.finish()
    }

    pub fn new_raw(gl: &Context, vertices: &[f32], indices: &[u32]) -> Result<Self, String> {
        unsafe {
            let vao = gl.create_vertex_array().map_err(|e| format!("Failed to create VAO: {}", e))?;
            gl.bind_vertex_array(Some(vao));

            let vbo = gl.create_buffer().map_err(|e| format!("Failed to create VBO: {}", e))?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                glow::STATIC_DRAW,
            );

            let ebo = gl.create_buffer().map_err(|e| format!("Failed to create EBO: {}", e))?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STATIC_DRAW,
            );

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 32, 0);

            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 32, 12);

            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 32, 24);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Ok(Mesh {
                inner: Arc::new(MeshInner {
                    vao,
                    vbo,
                    ebo,
                    indices_count: indices.len() as i32,
                }),
            })
        }
    }

    /// Create a mesh from raw terrain vertex data (stride = 72 bytes for TerrainVertex)
    /// TerrainVertex layout: position(3), normal(3), tangent(3), bitangent(3), texcoord(2), splat_weights(4)
    pub fn new_terrain(gl: &Context, vertices: &[f32], indices: &[u32]) -> Result<Self, String> {
        unsafe {
            let vao = gl.create_vertex_array().map_err(|e| format!("Failed to create VAO: {}", e))?;
            gl.bind_vertex_array(Some(vao));

            let vbo = gl.create_buffer().map_err(|e| format!("Failed to create VBO: {}", e))?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                glow::STATIC_DRAW,
            );

            let ebo = gl.create_buffer().map_err(|e| format!("Failed to create EBO: {}", e))?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STATIC_DRAW,
            );

            // Stride = 72 bytes (18 floats * 4 bytes)
            let stride: i32 = 72;
            
            // position: location 0, offset 0
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);

            // normal: location 1, offset 12
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 12);

            // tangent: location 2, offset 24
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 3, glow::FLOAT, false, stride, 24);

            // bitangent: location 3, offset 36
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_f32(3, 3, glow::FLOAT, false, stride, 36);

            // texcoord: location 4, offset 48
            gl.enable_vertex_attrib_array(4);
            gl.vertex_attrib_pointer_f32(4, 2, glow::FLOAT, false, stride, 48);

            // splat_weights: location 5, offset 56
            gl.enable_vertex_attrib_array(5);
            gl.vertex_attrib_pointer_f32(5, 4, glow::FLOAT, false, stride, 56);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Ok(Mesh {
                inner: Arc::new(MeshInner {
                    vao,
                    vbo,
                    ebo,
                    indices_count: indices.len() as i32,
                }),
            })
        }
    }

    pub fn draw(&self, gl: &Context) {
        // Пропускаем рендеринг для placeholder мешей с нулевым количеством индексов
        if self.inner.indices_count == 0 {
            return;
        }
        unsafe {
            gl.bind_vertex_array(Some(self.inner.vao));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.inner.ebo));
            gl.draw_elements(glow::TRIANGLES, self.inner.indices_count, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
        }
    }

    pub fn indices_count(&self) -> i32 {
        self.inner.indices_count
    }

    /// Явное удаление GPU-ресурса. Вызывать вручную перед уничтожением GL контекста.
    pub fn delete(&self, gl: &Context) {
        unsafe {
            // Проверяем, есть ли другие ссылки на этот меш
            if Arc::strong_count(&self.inner) == 1 {
                gl.delete_vertex_array(self.inner.vao);
                gl.delete_buffer(self.inner.vbo);
                gl.delete_buffer(self.inner.ebo);
            }
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        // Ресурсы удаляются только если это последняя ссылка
        // Для гарантированного удаления используйте метод delete(&self, gl: &Context)
    }
}
