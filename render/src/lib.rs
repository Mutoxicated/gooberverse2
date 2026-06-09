pub mod advanced_wire;
pub mod entity_renderer;

use std::{ffi::{CStr, CString, c_char, c_int, c_uint}, fmt::Display, os::raw::c_void};
use gl::{
    AttachShader, COMPILE_STATUS, CompileShader, CreateProgram, CreateShader, DeleteProgram, DeleteShader, FALSE, FRAGMENT_SHADER, GetProgramInfoLog, GetProgramiv, GetShaderInfoLog, GetShaderiv, GetUniformLocation, INFO_LOG_LENGTH, LINK_STATUS, LinkProgram, ShaderSource, Uniform1f, Uniform3fv, UniformMatrix4fv, UseProgram, VERTEX_SHADER
};
use gl::{
    ARRAY_BUFFER, BindBuffer, BindVertexArray, BufferData, DeleteBuffers, DeleteVertexArrays, DrawElements, ELEMENT_ARRAY_BUFFER, EnableVertexAttribArray, FLOAT, GenBuffers, GenVertexArrays, STATIC_DRAW, TRIANGLES, UNSIGNED_INT, VertexAttribPointer
};
use glam::{Mat4, Vec3};

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
    pub fn get_tuple_slice(&self) -> (&[f32], &[f32], &[c_uint], &[f32], &[f32]) {
        (
            self.vertices.as_slice(),
            self.colors.as_slice(),
            self.indices.as_slice(),
            self.normals.as_slice(),
            self.barycentrics.as_slice(),
        )
    }

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

pub struct ObjectShader {
    pub shader: InnerObjectShader,
    pub info: Box<dyn ShaderInfo>
}

impl ObjectShader {
    pub fn new(info: Box<dyn ShaderInfo>, path:String) -> Self {
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

pub struct EntityRenderer {
    mesh: Mesh,

    vao: c_uint,
    vbo_v: c_uint,
    vbo_c: c_uint,
    vbo_n: c_uint,
    vbo_b: c_uint,
    ebo: c_uint,
}

impl Drop for EntityRenderer {
    fn drop(&mut self) {
        unsafe {
            DeleteVertexArrays(1, &self.vao);
            DeleteBuffers(1, &self.vbo_v);
            DeleteBuffers(1, &self.vbo_c);
            DeleteBuffers(1, &self.vbo_n);
            DeleteBuffers(1, &self.vbo_b);
            DeleteBuffers(1, &self.ebo);
        }
    }
}

impl EntityRenderer {
    pub fn init(mesh: Mesh) -> Self {
        let mut instance = Self {
            mesh,
            vao: 0,
            vbo_v: 0,
            vbo_c: 0,
            vbo_n: 0,
            vbo_b: 0,
            ebo: 0,
        };

        let (verts, colors, indices, normals, barys) = instance.mesh.get_tuple_slice();
        unsafe {
            GenBuffers(1, &mut instance.vbo_v as *mut c_uint);
            GenBuffers(1, &mut instance.vbo_c as *mut c_uint);
            GenBuffers(1, &mut instance.vbo_n as *mut c_uint);
            GenBuffers(1, &mut instance.vbo_b as *mut c_uint);
            GenBuffers(1, &mut instance.ebo as *mut c_uint);
            GenVertexArrays(1, &mut instance.vao as *mut c_uint);

            BindVertexArray(instance.vao);

            BindBuffer(ARRAY_BUFFER, instance.vbo_v);
            BufferData(
                ARRAY_BUFFER,
                size_of_val(verts) as isize,
                verts.as_ptr() as *const c_void,
                STATIC_DRAW,
            );
            VertexAttribPointer(
                0,
                3,
                FLOAT,
                FALSE,
                3 * size_of::<f32>() as c_int,
                std::ptr::null(),
            );

            BindBuffer(ARRAY_BUFFER, instance.vbo_c);
            BufferData(
                ARRAY_BUFFER,
                size_of_val(colors) as isize,
                colors.as_ptr() as *const c_void,
                STATIC_DRAW,
            );
            VertexAttribPointer(
                1,
                4,
                FLOAT,
                FALSE,
                4 * size_of::<f32>() as c_int,
                std::ptr::null(),
            );

            BindBuffer(ARRAY_BUFFER, instance.vbo_n);
            BufferData(
                ARRAY_BUFFER,
                size_of_val(normals) as isize,
                normals.as_ptr() as *const c_void,
                STATIC_DRAW,
            );
            VertexAttribPointer(
                2,
                3,
                FLOAT,
                FALSE,
                3 * size_of::<f32>() as c_int,
                std::ptr::null(),
            );

            BindBuffer(ARRAY_BUFFER, instance.vbo_b);
            BufferData(
                ARRAY_BUFFER,
                size_of_val(barys) as isize,
                barys.as_ptr() as *const c_void,
                STATIC_DRAW,
            );
            VertexAttribPointer(
                3,
                3,
                FLOAT,
                FALSE,
                3 * size_of::<f32>() as c_int,
                std::ptr::null(),
            );

            EnableVertexAttribArray(0);
            EnableVertexAttribArray(1);
            EnableVertexAttribArray(2);
            EnableVertexAttribArray(3);

            BindBuffer(ELEMENT_ARRAY_BUFFER, instance.ebo);
            BufferData(
                ELEMENT_ARRAY_BUFFER,
                size_of_val(indices) as isize,
                indices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );
        }

        instance
    }

    pub fn draw(&self) {
        unsafe {
            BindVertexArray(self.vao);
            DrawElements(
                TRIANGLES,
                self.mesh.indices.len() as c_int,
                UNSIGNED_INT,
                std::ptr::null(),
            );
            BindVertexArray(0);
        }
    }
}
