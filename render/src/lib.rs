pub mod advanced_wire;
pub mod entity_renderer;
pub mod mesh;
pub use entity_renderer::EntityRenderer;

use gl::{
    AttachShader, COMPILE_STATUS, CompileShader, CreateProgram, CreateShader, DeleteProgram,
    DeleteShader, FALSE, FRAGMENT_SHADER, GetProgramInfoLog, GetProgramiv, GetShaderInfoLog,
    GetShaderiv, GetUniformLocation, INFO_LOG_LENGTH, LINK_STATUS, LinkProgram, ShaderSource,
    Uniform1f, Uniform3fv, UniformMatrix4fv, UseProgram, VERTEX_SHADER,
};
use glam::{Mat4, Vec3};
use std::{
    any::TypeId,
    ffi::{CStr, CString, c_char, c_int, c_uint},
    fmt::{Debug, Display},
};

pub mod uniform {
    ///mat4
    pub const PROJ: &str = "proj";
    ///mat4
    pub const VIEW: &str = "view";
    ///mat4
    pub const MODEL: &str = "model";
    ///float
    pub const SCALE: &str = "scale";
}

#[non_exhaustive]
#[derive(Debug)]
pub struct GraphicsError(String);

impl Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("Graphics Error: {}", self.0).as_str())
    }
}

pub struct ObjectShader {
    pub shader: InnerObjectShader,
    pub info: Box<dyn ShaderInfo + Send>,
}

impl ObjectShader {
    pub fn new(info: Box<dyn ShaderInfo + Send>, path: String) -> Self {
        let shader = InnerObjectShader::new(info.name(), path).unwrap();
        Self { shader, info }
    }
}

macro_rules! graphics_error {
    ($msg:expr $(, $arg:expr),*) => {
        GraphicsError(format!($msg $(,$arg),*))
    };
}

pub struct SpecialUnis {
    pub cam_pos: Vec3,
}

pub trait ShaderInfo: Send + 'static {
    fn name(&self) -> &str;
    fn set_special_uniforms(&self, e: &SpecialUnis, shader: &InnerObjectShader);
}

pub struct InnerObjectShader {
    prog: c_uint,
}

impl Drop for InnerObjectShader {
    fn drop(&mut self) {
        unsafe {
            DeleteProgram(self.prog);
        }
    }
}

impl InnerObjectShader {
    pub fn new(file_name: &str, dir: String) -> Result<Self, GraphicsError> {
        let dir = dir + file_name;
        let vertex_source = std::fs::read_to_string(dir.clone() + ".vert");
        if let Err(x) = vertex_source {
            return Err(graphics_error!(
                "Couldn't load vertex shader at '{dir}' ('{x}')"
            ));
        }
        let vertex_source = CString::new(vertex_source.unwrap()).unwrap();
        let vertex_source_ptr = vertex_source.as_ptr();

        let fragment_source = std::fs::read_to_string(dir.clone() + ".frag");
        if let Err(x) = fragment_source {
            return Err(graphics_error!(
                "Couldn't load vertex shader at '{dir}' ('{x}')"
            ));
        }
        let fragment_source = CString::new(fragment_source.unwrap()).unwrap();
        let fragment_source_ptr = fragment_source.as_ptr();

        let prg = unsafe {
            let vs: c_uint = CreateShader(VERTEX_SHADER);
            ShaderSource(vs, 1, &vertex_source_ptr, std::ptr::null());
            CompileShader(vs);
            let mut success: c_int = 0;
            GetShaderiv(vs, COMPILE_STATUS, &mut success as *mut c_int);
            if success == 0 {
                GetShaderiv(vs, INFO_LOG_LENGTH, &mut success as *mut c_int);
                let mut log: Vec<c_char> = vec![0; success as usize];
                GetShaderInfoLog(vs, success, std::ptr::null_mut(), log.as_mut_ptr());
                let msg = CStr::from_ptr(log.as_ptr())
                    .to_owned()
                    .to_string_lossy()
                    .to_string();
                return Err(graphics_error!(
                    "Failed to compile the vertex shader! Message: {msg}"
                ));
            }

            let fs: c_uint = CreateShader(FRAGMENT_SHADER);
            ShaderSource(fs, 1, &fragment_source_ptr, std::ptr::null());
            CompileShader(fs);
            let mut success: c_int = 0;
            GetShaderiv(fs, COMPILE_STATUS, &mut success as *mut c_int);
            if success == 0 {
                GetShaderiv(vs, INFO_LOG_LENGTH, &mut success as *mut c_int);
                let mut log: Vec<c_char> = vec![0; success as usize];
                GetShaderInfoLog(vs, success, std::ptr::null_mut(), log.as_mut_ptr());
                let msg = CStr::from_ptr(log.as_ptr())
                    .to_owned()
                    .to_string_lossy()
                    .to_string();
                return Err(graphics_error!(
                    "Failed to compile the fragment shader! Message: {msg}"
                ));
            }

            let prog: c_uint = CreateProgram();
            AttachShader(prog, vs);
            AttachShader(prog, fs);
            LinkProgram(prog);
            let mut success: c_int = 0;
            GetProgramiv(prog, LINK_STATUS, &mut success as *mut c_int);
            if success == 0 {
                GetProgramiv(prog, INFO_LOG_LENGTH, &mut success as *mut c_int);
                let mut log: Vec<c_char> = vec![0; success as usize];
                GetProgramInfoLog(prog, success, std::ptr::null_mut(), log.as_mut_ptr());
                let msg = CStr::from_ptr(log.as_ptr())
                    .to_owned()
                    .to_string_lossy()
                    .to_string();
                return Err(graphics_error!("Failed to link program! Message: {msg}"));
            }

            DeleteShader(vs);
            DeleteShader(fs);

            prog
        };
        Ok(Self { prog: prg })
    }

    pub fn activate(&self) {
        unsafe {
            UseProgram(self.prog);
        }
    }

    pub fn set_float(&self, name: &str, f: f32) {
        unsafe {
            let cstr = CString::new(name).unwrap();
            let a = GetUniformLocation(self.prog, cstr.as_ptr());
            if a == -1 {
                return;
            }
            Uniform1f(a, f);
        }
    }

    pub fn set_vec3(&self, name: &str, v: Vec3) {
        unsafe {
            let cstr = CString::new(name).unwrap();
            let a = GetUniformLocation(self.prog, cstr.as_ptr());
            if a == -1 {
                return;
            }
            Uniform3fv(a, 1, v.as_ref().as_ptr());
        }
    }

    pub fn set_mat4(&self, name: &str, mat: Mat4) {
        unsafe {
            let cstr = CString::new(name).unwrap();
            let a = GetUniformLocation(self.prog, cstr.as_ptr());
            if a == -1 {
                return;
            }
            UniformMatrix4fv(a, 1, FALSE, mat.as_ref().as_ptr());
        }
    }
}

pub struct RenderObject {
    pub model_matrix: Mat4,
    pub entity_type_id: TypeId,
    pub shaders_to_use: Vec<u8>,
}
