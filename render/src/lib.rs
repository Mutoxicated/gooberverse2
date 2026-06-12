pub mod advanced_wire;
pub mod entity_renderer;

use std::{ffi::{CStr, CString, c_char, c_int, c_uint}, fmt::Display};
use gl::{
    AttachShader, COMPILE_STATUS, CompileShader, CreateProgram, CreateShader, DeleteProgram, DeleteShader, FALSE, FRAGMENT_SHADER, GetProgramInfoLog, GetProgramiv, GetShaderInfoLog, GetShaderiv, GetUniformLocation, INFO_LOG_LENGTH, LINK_STATUS, LinkProgram, ShaderSource, Uniform1f, Uniform3fv, UniformMatrix4fv, UseProgram, VERTEX_SHADER
};
use glam::{Mat4, Vec3};

pub use entity_renderer::EntityRenderer;

#[derive(PartialEq)]
pub enum WireType {
    /// Displays a classic wireframe
    Triangle,
    /// Displays a quad wireframe
    Quad,
    /// Displays a wireframe that hides lines between triangles with almost the same surface normal
    Advanced
}

pub struct MeshBuilder {
    vertices: Vec<f32>,
    colors: Vec<f32>,
    #[allow(dead_code)]
    indices: Vec<c_uint>,
    normals: Vec<f32>
}

impl MeshBuilder {
    pub fn with_verts(verts: Vec<f32>) -> Self {
        if verts.len() < 9 {
            println!("Invalid mesh vertices. It won't be renderable because it doesn't have enough vertices for at least one triangle.");
            return Self { vertices: verts, colors: Vec::new(), indices: Vec::new(), normals: Vec::new() }
        }

        Self {
            vertices: verts,
            colors: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new()
            
        }
    }

    pub fn with_colors(mut self, colors: Vec<f32>) -> Self {
        if colors.len()/4 != self.vertices.len()/3 {
            println!("Invalid mesh data. Amount of colors != Amount of vertices. (Maybe you didn't give the alpha channels?)")
        }
        self.colors = colors;
        self
    }

    pub fn with_indices(mut self, indices: Vec<c_uint>) -> Mesh {
        if indices.len() < 3 {
            println!("Invalid mesh data. Indices must be more than 2 to form at least 1 triangle.");

        }else if indices.len() % 3 != 0 {
            println!("Invalid mesh data. Indices must be in pairs of 3.");
        }
        let mut new_verts:Vec<f32> = Vec::new();
        let mut new_colors:Vec<f32> = Vec::new();
        let mut i = 0;
        while i <= indices.len() - 3 {
            let a = indices[i] as usize *3;
            let one = Vec3::new(self.vertices[a], self.vertices[a+1], self.vertices[a+2]);
            let b = indices[i+1] as usize *3;
            let two = Vec3::new(self.vertices[b], self.vertices[b+1], self.vertices[b+2]);
            let c = indices[i+2] as usize *3;
            let three = Vec3::new(self.vertices[c], self.vertices[c+1], self.vertices[c+2]);

            let vector1 = two-one;
            let vector2 = three-one;

            let mut normal = vector1.cross(vector2).normalize();

            if normal.dot(three+one+two).is_sign_negative() {
                normal = vector2.cross(vector1).normalize();
            }
            
            self.normals.extend_from_slice(&[normal.x, normal.y, normal.z, normal.x, normal.y, normal.z, normal.x, normal.y, normal.z]);
            new_verts.extend_from_slice(&[one.x, one.y, one.z, two.x, two.y, two.z, three.x, three.y, three.z]);
            let a = indices[i] as usize * 4;
            let b = indices[i+1] as usize * 4;
            let c = indices[i+2] as usize * 4;
            new_colors.extend_from_slice(&[
                self.colors[a], self.colors[a+1], self.colors[a+2], self.colors[a+3],
                self.colors[b], self.colors[b+1], self.colors[b+2], self.colors[b+3],
                self.colors[c], self.colors[c+1], self.colors[c+2], self.colors[c+3],
            ]);

            i += 3;
        }
        let new_indices:Vec<c_uint> = (0..(new_verts.len()/3) as u32).collect();
        let new_verts_len = new_verts.len();
        Mesh {
            vertices: new_verts,
            colors: new_colors,
            indices: new_indices,
            normals: self.normals,
            barycentrics: [
                0.1, 0.0, 0.0,
                0.0, 0.1, 0.0,
                0.0, 0.0, 0.1,
            ].repeat(new_verts_len/3)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<c_uint>,
    pub normals: Vec<f32>,
    pub barycentrics: Vec<f32>
}

impl Mesh {
    pub fn bake_wireframe(&mut self, wire_type: WireType) {
        if wire_type == WireType::Triangle {
            return;
        }
        if wire_type == WireType::Advanced {
            self.barycentrics = advanced_wire::calculate_barycentrics_adv(&self.vertices, &self.normals);
            return;
        }
        self.barycentrics = advanced_wire::calculate_barycentrics_quad(&self.vertices);
    } 
}

#[non_exhaustive]
#[derive(Debug)]
pub struct GraphicsError(String);

impl Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("Graphics Error: {}", self.0).as_str())
    }
}

/// # IMPORTANT NOTE

/// A type `T` is `Send`, given that when a value of type `T` is owned by **two** different 
/// threads, any manipulation of that data will not lead to UB.
///
/// **Implementing the `Send` trait, as a user of this type I hereby promise that 
/// any value of type `ObjectShader` will NEVER be co-owned by more than 1 thread, 
/// and therefore will NEVER be manipulated by more than 1 thread, and so it will 
/// ALWAYS be `Send` -able.**
/// 
/// - UB Prevention Services
pub struct ObjectShader {
    pub shader: InnerObjectShader,
    pub info: Box<dyn ShaderInfo + Send>
}

impl ObjectShader {
    pub fn new(info: Box<dyn ShaderInfo + Send>, path:String) -> Self {
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

pub trait ShaderInfo {
    fn name(&self) -> &str;
    fn set_special_uniforms(&self, e:&SpecialUnis, shader: &InnerObjectShader);
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
    pub fn new(file_name:&str, dir: String) -> Result<Self, GraphicsError> {
        let dir = dir+file_name;
        let vertex_source = std::fs::read_to_string(dir.clone()+".vert");
        if let Err(x) = vertex_source {
            return Err(graphics_error!("Couldn't load vertex shader at '{dir}' ('{x}')"));
        }
        let vertex_source = CString::new(vertex_source.unwrap()).unwrap();
        let vertex_source_ptr = vertex_source.as_ptr();

        let fragment_source = std::fs::read_to_string(dir.clone()+".frag");
        if let Err(x) = fragment_source {
            return Err(graphics_error!("Couldn't load vertex shader at '{dir}' ('{x}')"));
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
                let mut log: Vec<c_char> = Vec::with_capacity(success as usize);
                GetShaderInfoLog(vs, success, std::ptr::null_mut(), log.as_mut_ptr());
                let msg = CStr::from_ptr(log.as_ptr())
                    .to_owned()
                    .to_string_lossy()
                    .to_string();
                return Err(graphics_error!("Failed to compile the vertex shader! Message: {msg}"));
            }

            let fs: c_uint = CreateShader(FRAGMENT_SHADER);
            ShaderSource(fs, 1, &fragment_source_ptr, std::ptr::null());
            CompileShader(fs);
            let mut success: c_int = 0;
            GetShaderiv(fs, COMPILE_STATUS, &mut success as *mut c_int);
            if success == 0 {
                GetShaderiv(vs, INFO_LOG_LENGTH, &mut success as *mut c_int);
                let mut log: Vec<c_char> = Vec::with_capacity(success as usize);
                GetShaderInfoLog(vs, success, std::ptr::null_mut(), log.as_mut_ptr());
                let msg = CStr::from_ptr(log.as_ptr())
                    .to_owned()
                    .to_string_lossy()
                    .to_string();
                return Err(graphics_error!("Failed to compile the fragment shader! Message: {msg}"));
            }

            let prog: c_uint = CreateProgram();
            AttachShader(prog, vs);
            AttachShader(prog, fs);
            LinkProgram(prog);
            let mut success: c_int = 0;
            GetProgramiv(prog, LINK_STATUS, &mut success as *mut c_int);
            if success == 0 {
                GetProgramiv(prog, INFO_LOG_LENGTH, &mut success as *mut c_int);
                let mut log: Vec<c_char> = Vec::with_capacity(success as usize);
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
        Ok(Self { prog:prg })
    }

    pub fn activate(&self) {
        unsafe{ UseProgram(self.prog); }
    }

    pub fn set_float(&self, name:&str, f:f32) {
        unsafe {
            let cstr = CString::new(name).unwrap();
            let a = GetUniformLocation(self.prog, cstr.as_ptr());
            if a == -1 {
                return;
            }
            Uniform1f(a, f);
        }
    }

    pub fn set_vec3(&self, name:&str, v:Vec3) {
        unsafe {
            let cstr = CString::new(name).unwrap();
            let a = GetUniformLocation(self.prog, cstr.as_ptr());
            if a == -1 {
                return;
            }
            Uniform3fv(a, 1, v.as_ref().as_ptr());
        }
    }

    pub fn set_mat4(&self, name:&str, mat:Mat4) {
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
