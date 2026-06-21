use std::{ffi::c_uint, fmt::Debug};

use glam::Vec3;

use crate::advanced_wire;

#[derive(PartialEq, Clone)]
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
        let c_amount = colors.len() / 4;
        let v_amount = self.vertices.as_ref().unwrap().len() / 3;
        if c_amount != v_amount {
            println!(
                "Invalid mesh data. Amount of colors ({c_amount}) != Amount of  ({v_amount}). Maybe you didn't give the alpha channels?"
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
        } else if !indices.len().is_multiple_of(3) {
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
    pub(crate) _invalid: bool,
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

pub enum ExtraOptions {
    Nothing,
    BakeWireframe(WireType),
}

pub enum MeshFileType {
    GLTF,
}

impl MeshFileType {
    pub fn as_str(&self) -> &str {
        match *self {
            MeshFileType::GLTF => ".gltf",
        }
    }
}

pub enum LoadMeshError {
    IOError(std::io::Error),
    GLTFError(gltf::Error),
}

impl Debug for LoadMeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut msg = String::from("Load Mesh Error! ");
        match self {
            LoadMeshError::IOError(x) => {
                msg.push_str(format!("{:?}", x).as_str());
            }
            LoadMeshError::GLTFError(x) => {
                msg.push_str(format!("{:?}", x).as_str());
            }
        }

        f.write_str(msg.as_str())
    }
}

pub struct MeshAsset {
    name: Box<str>,
    file_type: MeshFileType,
    extra: ExtraOptions,
}

impl MeshAsset {
    pub fn new(name: &str, r#type: MeshFileType, extra: ExtraOptions) -> Self {
        Self {
            name: name.into(),
            file_type: r#type,
            extra,
        }
    }

    pub fn load_mesh(&self) -> Result<Mesh, LoadMeshError> {
        // use MeshFileType as M;
        use ExtraOptions as E;

        let cwd = std::env::current_dir();
        if let Err(x) = cwd {
            return Err(LoadMeshError::IOError(x));
        }
        let cwd = cwd.unwrap();

        let mut p = String::from(self.name.clone());
        p.push_str(self.file_type.as_str());
        let file_path = cwd.join(p);
        //println!("{}", file_path.to_str().unwrap());
        let res = gltf::import(file_path);
        if let Err(x) = res {
            return Err(LoadMeshError::GLTFError(x));
        }
        let (doc, buffers, _) = res.unwrap();
        let mut vertices = Vec::<f32>::new();
        let mut indices = Vec::<c_uint>::new();
        let mut colors = Vec::<f32>::new();
        for mesh in doc.meshes() {
            let mut read_offset = 0;
            for p in mesh.primitives() {
                let r = p.reader(|buffer| Some(&buffers[buffer.index()]));

                let mut add_offset = 0;
                if let Some(a) = r.read_positions() {
                    for position in a {
                        vertices.extend_from_slice(&position);
                        add_offset += 1;
                    }
                }

                if let Some(a) = r.read_indices() {
                    for index in a.into_u32() {
                        indices.push(read_offset + index as c_uint);
                    }
                }

                if let Some(a) = r.read_colors(4) {
                    for c in a.into_rgba_f32() {
                        colors.extend_from_slice(&c);
                    }
                }

                read_offset += add_offset;
            }
        }

        let c_amount = colors.len() / 4;
        let v_amount = vertices.len() / 3;
        if c_amount < v_amount {
            colors.extend([1.0, 1.0, 1.0, 1.0].repeat(v_amount - c_amount).iter());
        }

        let mut mesh = MeshBuilder::builder(vertices)
            .with_colors(colors)
            .with_indices(indices)
            .build();
        match &self.extra {
            E::BakeWireframe(x) => {
                mesh = mesh.bake_wireframe(x.clone());
            }
            E::Nothing => {}
        }
        Ok(mesh)
    }
}
