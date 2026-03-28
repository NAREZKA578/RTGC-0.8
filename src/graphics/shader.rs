use glow::{Context, HasContext};

pub struct Shader {
    program: glow::Program,
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        // Note: This creates a shallow clone - actual GPU resources are not duplicated
        Self {
            program: self.program,
        }
    }
}

impl std::fmt::Debug for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shader")
            .field("program", &"glow::Program")
            .finish()
    }
}

impl Shader {
    pub fn new(
        gl: &Context,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let vertex_shader = compile_shader(gl, glow::VERTEX_SHADER, vertex_shader_source)?;
            let fragment_shader = compile_shader(gl, glow::FRAGMENT_SHADER, fragment_shader_source)?;

            let program = gl.create_program()
                .map_err(|e| format!("Failed to create program: {}", e))?;
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                return Err(
                    format!("Failed to link shader program: {}", gl.get_program_info_log(program)).into()
                );
            }

            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);

            Ok(Shader { program })
        }
    }

    pub fn bind(&self, gl: &Context) {
        unsafe {
            gl.use_program(Some(self.program));
        }
    }

    pub fn unbind(gl: &Context) {
        unsafe {
            gl.use_program(None);
        }
    }

    pub fn program(&self) -> glow::Program {
        self.program
    }
}

unsafe fn compile_shader(
    gl: &Context,
    shader_type: u32,
    source: &str,
) -> Result<glow::Shader, Box<dyn std::error::Error>> {
    let shader = gl.create_shader(shader_type)
        .map_err(|e| format!("Failed to create shader: {}", e))?;
    gl.shader_source(shader, source);
    gl.compile_shader(shader);

    if !gl.get_shader_compile_status(shader) {
        return Err(format!("Failed to compile shader: {}", gl.get_shader_info_log(shader)).into());
    }

    Ok(shader)
}

impl Drop for Shader {
    fn drop(&mut self) {
        // Note: Shader program deletion requires GL context which is not available here.
        // In a real application, you would need to call explicit cleanup methods before dropping
        // or use a resource manager that tracks the GL context lifetime.
        // For now, resources are cleaned up when GL context is destroyed.
        // If explicit cleanup is needed, add a delete(&self, gl: &Context) method and call it manually.
    }
}
