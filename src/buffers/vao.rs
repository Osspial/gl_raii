use gl::{self, Gl};
use gl::types::*;

use super::{Vertex, Index, Buffer, BufferUsage};
use types::{GLSLType, GLPrim};

use std::mem;
use std::cell::Cell;
use std::marker::PhantomData;

pub struct VertexArrayObj<V: Vertex, I: Index> {
    handle: GLuint,
    vertex_buffer: Buffer<V>,
    index_buffer: Buffer<I>
}

pub struct VertexAttribBuilder<'a, V: Vertex> {
    attrib_index: u32,
    max_attribs: u32,
    gl: &'a Gl,
    _marker: PhantomData<V>
}

pub(crate) struct VertexArrayObjTarget {
    bound_vao: Cell<GLuint>,
    _sendsync_optout: PhantomData<*const ()>
}

pub(crate) struct BoundVAO<'a, V: Vertex, I: Index> {
    vao: &'a VertexArrayObj<V, I>
}


impl<V: Vertex, I: Index> VertexArrayObj<V, I> {
    pub fn new(vertex_buffer: Buffer<V>, index_buffer: Buffer<I>) -> VertexArrayObj<V, I> {
        if vertex_buffer.state.as_ref() as *const _ != index_buffer.state.as_ref() as *const _ {
            panic!("vertex buffer and index buffer using different contexts");
        }
        unsafe {
            let mut handle = 0;
            let mut max_attribs = 0;
            vertex_buffer.state.gl.GenVertexArrays(1, &mut handle);

            let vao = VertexArrayObj { handle, vertex_buffer, index_buffer };

            {
                let state = &vao.vertex_buffer.state;
                let vao_bind = state.buffer_binds.vao_bind.bind(&vao);
                vao_bind.init_bind();

                state.gl.GetIntegerv(gl::MAX_VERTEX_ATTRIBS, &mut max_attribs);
                let vab = VertexAttribBuilder {
                    attrib_index: 0,
                    max_attribs: max_attribs as u32,
                    gl: &state.gl,
                    _marker: PhantomData
                };
                V::register_attribs(vab);
            }

            vao
        }
    }
}

impl<V: Vertex> VertexArrayObj<V, ()> {
    #[inline]
    pub fn new_noindex(vertex_buffer: Buffer<V>) -> VertexArrayObj<V, ()> {
        let index_buffer: Buffer<()> = Buffer::with_size(BufferUsage::StaticDraw, 0, vertex_buffer.state.clone()).unwrap();
        VertexArrayObj::new(vertex_buffer, index_buffer)
    }
}

impl<V: Vertex, I: Index> Drop for VertexArrayObj<V, I> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let state = &self.vertex_buffer.state;
            state.gl.DeleteVertexArrays(1, &self.handle);
            if state.buffer_binds.vao_bind.bound_vao.get() == self.handle {
                state.buffer_binds.vao_bind.reset_bind(&state.gl);
            }
        }
    }
}

impl<'a, V: Vertex> VertexAttribBuilder<'a, V> {
    #[inline]
    pub fn add_vertex_attrib<T: GLSLType>(&mut self, name: &str, get_type: fn(&V) -> &T) {
        let gl = self.gl;
        let vertex = V::default();

        let attrib_ptr = get_type(&vertex) as *const T;
        let attrib_offset = attrib_ptr as *const u8 as isize - &vertex as *const V as *const u8 as isize;

        // Make sure the attribute is actually inside of the type, instead of pointing to a static or smth.
        assert!(attrib_offset >= 0);
        let attrib_offset = attrib_offset as usize;
        assert!(attrib_offset + mem::size_of::<T>() <= mem::size_of::<V>());

        let attrib_size = T::len() * mem::size_of::<T::GLPrim>();
        assert!(attrib_size <= mem::size_of::<T>());

        unsafe {
            if self.attrib_index < self.max_attribs {
                gl.EnableVertexAttribArray(self.attrib_index);
                if T::GLPrim::gl_enum() != gl::DOUBLE {
                    gl.VertexAttribPointer(
                        self.attrib_index,
                        T::len() as GLint,
                        T::GLPrim::gl_enum(),
                        T::GLPrim::normalized() as GLboolean,
                        mem::size_of::<V>() as GLsizei,
                        attrib_offset as *const GLvoid
                    );
                } else {
                    panic!("Attempting to use OpenGL 4 feature")
                    // gl.VertexAttribLPointer(
                    //     self.attrib_index,
                    //     T::len() as GLint,
                    //     T::GLPrim::gl_enum(),
                    //     mem::size_of::<V>() as GLsizei,
                    //     attrib_offset as *const GLvoid
                    // );
                }

                self.attrib_index += 1;
            } else {
                panic!(
                    "Too many attributes on field {}; GL implementation has maximum of {}",
                    name,
                    self.max_attribs
                );
            }
            assert_eq!(0, gl.GetError());
        }
    }
}

impl VertexArrayObjTarget {
    #[inline]
    pub fn new() -> VertexArrayObjTarget {
        VertexArrayObjTarget {
            bound_vao: Cell::new(0),
            _sendsync_optout: PhantomData
        }
    }

    #[inline]
    pub unsafe fn bind<'a, V: Vertex, I: Index>(&'a self, vao: &'a VertexArrayObj<V, I>) -> BoundVAO<'a, V, I> {
        if self.bound_vao.get() != vao.handle {
            let gl = &vao.vertex_buffer.state.gl;
            gl.BindVertexArray(vao.handle);
            self.bound_vao.set(vao.handle);
        }
        BoundVAO {
            vao
        }
    }

    #[inline]
    pub unsafe fn reset_bind(&self, gl: &Gl) {
        gl.BindVertexArray(0);
    }
}

impl<'a, V: Vertex, I: Index> BoundVAO<'a, V, I> {
    /// Perform the initial setup involved with the VAO and bind the vertex and element array
    /// buffers
    #[inline]
    fn init_bind(&self) {
        unsafe {
            let gl = &self.vao.vertex_buffer.state.gl;
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vao.vertex_buffer.raw.handle());
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.vao.index_buffer.raw.handle());
            assert_eq!(0, gl.GetError());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::CONTEXT_STATE;
    use quickcheck::{Arbitrary, Gen};

    #[derive(Debug, Default, Clone, Copy)]
    struct TestVertex {
        vert: [f32; 3],
        color: [f32; 3]
    }

    impl Vertex for TestVertex {
        fn register_attribs(mut attrib_builder: VertexAttribBuilder<Self>) {
            attrib_builder.add_vertex_attrib("vert", |t| &t.vert);
            attrib_builder.add_vertex_attrib("color", |t| &t.color);
        }
    }

    impl Arbitrary for TestVertex {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            TestVertex {
                vert: [f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g)],
                color: [f32::arbitrary(g), f32::arbitrary(g), f32::arbitrary(g)]
            }
        }
    }

    quickcheck!{
        fn make_vao_noindex(buffer_data: Vec<TestVertex>) -> () {
            CONTEXT_STATE.with(|context_state| {
                let vertex_buffer = Buffer::with_data(BufferUsage::StaticDraw, &buffer_data, context_state.clone()).unwrap();
                let _vao = VertexArrayObj::new_noindex(vertex_buffer);
            });
        }
    }
}