use glow::{Context, HasContext};
use std::sync::Arc;

pub struct TextureInner {
    texture: glow::Texture,
}

#[derive(Clone)]
pub struct Texture {
    inner: Arc<TextureInner>,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("texture", &"glow::Texture")
            .finish()
    }
}

impl Texture {
    pub fn new(gl: &Context, data: &[u8], width: u32, height: u32) -> Result<Self, String> {
        unsafe {
            let texture = gl
                .create_texture()
                .map_err(|e| format!("Failed to create texture: {}", e))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                width as i32,
                height as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(data),
            );

            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);

            Ok(Texture {
                inner: Arc::new(TextureInner { texture }),
            })
        }
    }

    /// Create a placeholder texture (for async loading)
    /// SAFETY: Uses NonZero with value 1 as dummy handle - caller must replace with real texture
    #[deprecated(note = "Placeholder texture will be replaced by async loader")]
    pub fn new_placeholder() -> Result<Self, String> {
        use std::num::NonZero;
        Ok(Self {
            inner: Arc::new(TextureInner {
                texture: glow::NativeTexture(unsafe { NonZero::new_unchecked(1) }),
            }),
        })
    }

    pub fn from_rgba8(gl: &Context, width: u32, height: u32, data: &[u8]) -> Result<Self, String> {
        unsafe {
            let texture = gl
                .create_texture()
                .map_err(|e| format!("Failed to create texture: {}", e))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(data),
            );

            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);

            Ok(Texture {
                inner: Arc::new(TextureInner { texture }),
            })
        }
    }

    pub fn bind(&self, gl: &Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.inner.texture));
        }
    }

    pub fn unbind(gl: &Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Явное удаление GPU-ресурса. Вызывать вручную перед уничтожением GL контекста.
    pub fn delete(&self, gl: &Context) {
        unsafe {
            // Проверяем, есть ли другие ссылки на эту текстуру
            if Arc::strong_count(&self.inner) == 1 {
                gl.delete_texture(self.inner.texture);
            }
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        // Resources are deleted when the last reference is dropped
        // The actual GL context must still be alive for this to work safely
        // In practice, textures should be explicitly deleted before destroying the GL context
        if Arc::strong_count(&self.inner) == 1 {
            // We can't delete GL resources here without access to the GL context
            // This is a limitation of OpenGL - resources are context-bound
            // Use texture.delete(&gl) explicitly before context destruction
        }
    }
}
