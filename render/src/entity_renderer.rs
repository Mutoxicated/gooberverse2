use {
    crate::Mesh,
    gl::{
        ARRAY_BUFFER, BindBuffer, BindVertexArray, BufferData, DeleteBuffers, DeleteVertexArrays,
        DrawElements, ELEMENT_ARRAY_BUFFER, EnableVertexAttribArray, FALSE, FLOAT, GenBuffers,
        GenVertexArrays, STATIC_DRAW, TRIANGLES, UNSIGNED_INT, VertexAttribPointer,
    },
    std::{
        ffi::{c_int, c_uint},
        os::raw::c_void,
    },
};

#[derive(Clone)]
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

        let verts = instance.mesh.vertices.as_slice();
        let colors = instance.mesh.colors.as_slice();
        let indices = instance.mesh.indices.as_slice();
        let normals = instance.mesh.normals.as_slice();
        let barys = instance.mesh.barycentrics.as_slice();

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
