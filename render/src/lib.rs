pub mod advanced_wire;
pub mod entity_renderer;
pub use entity_renderer::EntityRenderer;

use gl::{
    AttachShader, COMPILE_STATUS, CompileShader, CreateProgram, CreateShader, DeleteProgram,
    DeleteShader, FALSE, FRAGMENT_SHADER, GetProgramInfoLog, GetProgramiv, GetShaderInfoLog,
    GetShaderiv, GetUniformLocation, INFO_LOG_LENGTH, LINK_STATUS, LinkProgram, ShaderSource,
    Uniform1f, Uniform3fv, UniformMatrix4fv, UseProgram, VERTEX_SHADER,
};
use glam::{Mat4, Vec3};
use std::{
    ffi::{CStr, CString, c_char, c_int, c_uint},
    fmt::Display,
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

#[derive(PartialEq)]
pub enum WireType {
    /// Displays a classic wireframe
    Triangle,
    /// Displays a quad wireframe
    Quad,
    /// Displays a wireframe that hides lines between triangles with almost the same surface normal
    Advanced,
}

/// A builder for building meshes.A
///
/// The correct way to use it is like so:
/// ```
/// let mesh = MeshBuilder::builder(vec![-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0])
///     .with_colors(vec![0.4, 0.0, 0.0, 1.0, 0.4, 0.0, 0.9, 1.0, 0.0, 1.0, 0.0, 1.0, 0.1, 0.3, 1.0, 1.0 ])
///     .with_indices(vec![0,1,2,3,0])
///     .build();
///
/// // or like this
/// let mesh = MeshBuilder::builder(vec![-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0])
///     .with_indices(vec![0,1,2,3,0])
///     .with_colors(vec![0.4, 0.0, 0.0, 1.0, 0.4, 0.0, 0.9, 1.0, 0.0, 1.0, 0.0, 1.0, 0.1, 0.3, 1.0, 1.0 ])
///     .build();
/// ```
#[derive(Default)]
pub struct MeshBuilder {
    vertices: Option<Vec<f32>>,
    colors: Option<Vec<f32>>,
    indices: Option<Vec<c_uint>>,
    normals: Option<Vec<f32>>,
    _invalid: bool,
}

impl MeshBuilder {
    pub fn builder(verts: Vec<f32>) -> Self {
        let mut a = Self::default();
        if verts.len() < 9 {
            println!(
                "Invalid mesh vertices. It won't be renderable because it doesn't have enough vertices for at least one triangle."
            );
            a._invalid = true;
        }
        a.vertices = Some(verts);
        a
    }

    pub fn with_colors(mut self, colors: Vec<f32>) -> Self {
        if colors.len() / 4 != self.vertices.as_ref().unwrap().len() / 3 {
            println!(
                "Invalid mesh data. Amount of colors != Amount of vertices. (Maybe you didn't give the alpha channels?)"
            );
            self._invalid = true;
        }
        self.colors = Some(colors);
        self
    }

    pub fn with_indices(mut self, indices: Vec<c_uint>) -> Self {
        if indices.len() < 3 {
            println!("Invalid mesh data. Indices must be more than 2 to form at least 1 triangle.");
            self._invalid = true;
        } else if indices.len() % 3 != 0 {
            println!("Invalid mesh data. Indices must be in pairs of 3.");
            self._invalid = true;
        }
        self.indices = Some(indices);
        self
    }

    pub fn build(mut self) -> Mesh {
        assert_ne!(self.colors, None);
        assert_ne!(self.indices, None);

        let indices = self.indices.unwrap();
        let colors = self.colors.unwrap();
        let vertices = self.vertices.unwrap();
        self.normals = Some(vec![]);

        let mut new_verts: Vec<f32> = Vec::new();
        let mut new_colors: Vec<f32> = Vec::new();
        let mut i = 0;
        while i <= indices.len() - 3 {
            let a = indices[i] as usize * 3;
            let one = Vec3::new(vertices[a], vertices[a + 1], vertices[a + 2]);
            let b = indices[i + 1] as usize * 3;
            let two = Vec3::new(vertices[b], vertices[b + 1], vertices[b + 2]);
            let c = indices[i + 2] as usize * 3;
            let three = Vec3::new(vertices[c], vertices[c + 1], vertices[c + 2]);

            let vector1 = two - one;
            let vector2 = three - one;

            let mut normal = vector1.cross(vector2).normalize();

            if normal.dot(three + one + two).is_sign_negative() {
                normal = vector2.cross(vector1).normalize();
            }

            self.normals.as_mut().unwrap().extend_from_slice(&[
                normal.x, normal.y, normal.z, normal.x, normal.y, normal.z, normal.x, normal.y,
                normal.z,
            ]);
            new_verts.extend_from_slice(&[
                one.x, one.y, one.z, two.x, two.y, two.z, three.x, three.y, three.z,
            ]);
            let a = indices[i] as usize * 4;
            let b = indices[i + 1] as usize * 4;
            let c = indices[i + 2] as usize * 4;
            new_colors.extend_from_slice(&[
                colors[a],
                colors[a + 1],
                colors[a + 2],
                colors[a + 3],
                colors[b],
                colors[b + 1],
                colors[b + 2],
                colors[b + 3],
                colors[c],
                colors[c + 1],
                colors[c + 2],
                colors[c + 3],
            ]);

            i += 3;
        }
        let new_indices: Vec<c_uint> = (0..(new_verts.len() / 3) as u32).collect();
        let new_verts_len = new_verts.len();
        Mesh {
            vertices: new_verts,
            colors: new_colors,
            indices: new_indices,
            normals: self.normals.unwrap(),
            barycentrics: [0.1, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.1].repeat(new_verts_len / 3),
            _invalid: self._invalid,
        }
    }
}

/// A mesh.
/// A color value is in the range [0, 1.0]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<c_uint>,
    pub normals: Vec<f32>,
    pub barycentrics: Vec<f32>,
    _invalid: bool,
}

impl Mesh {
    pub const EMPTY: Self = Self {
        vertices: vec![],
        colors: vec![],
        indices: vec![],
        normals: vec![],
        barycentrics: vec![],
        _invalid: true,
    };

    pub fn bake_wireframe(mut self, wire_type: WireType) -> Self {
        if wire_type == WireType::Triangle {
            return self;
        }
        if wire_type == WireType::Advanced {
            self.barycentrics =
                advanced_wire::calculate_barycentrics_adv(&self.vertices, &self.normals);
            return self;
        }
        self.barycentrics = advanced_wire::calculate_barycentrics_quad(&self.vertices);
        self
    }

    pub fn is_invalid(&self) -> bool {
        self._invalid
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
    pub entity_id: u64,
    pub shaders_to_use: Vec<u8>,
}
