use glow::{Context, HasContext};

pub struct Texture {
    texture: glow::Texture,
}

impl Texture {
    pub fn new(gl: &Context, data: &[u8], width: u32, height: u32) -> Result<Self, String> {
        unsafe {
            let texture = gl.create_texture().map_err(|e| format!("Failed to create texture: {}", e))?;
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
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.bind_texture(glow::TEXTURE_2D, None);

            Ok(Texture { texture })
        }
    }

    pub fn from_rgba8(gl: &Context, width: u32, height: u32, data: &[u8]) -> Result<Self, String> {
        unsafe {
            let texture = gl.create_texture().map_err(|e| format!("Failed to create texture: {}", e))?;
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
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.bind_texture(glow::TEXTURE_2D, None);

            Ok(Texture { texture })
        }
    }

    pub fn bind(&self, gl: &Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
        }
    }

    pub fn unbind(gl: &Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        // Resources cleaned up with context
    }
}
