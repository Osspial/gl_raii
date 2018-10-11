// Copyright 2018 Osspial
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use {Handle, ContextState, GLObject};
mod raw;
use self::raw::{RawRenderbuffer, RawRenderbufferTarget};
use image_format::{UncompressedFormat, GLFormat};

use cgmath::Point2;
use cgmath_geometry::DimsBox;
use std::mem;
use std::rc::Rc;
use std::marker::PhantomData;

pub(crate) struct RenderbufferTarget(RawRenderbufferTarget);
pub struct Renderbuffer<I: UncompressedFormat> {
    raw: RawRenderbuffer,
    samples: u32,
    dims: DimsBox<Point2<u32>>,
    state: Rc<ContextState>,
    _format: PhantomData<I>
}

impl RenderbufferTarget {
    pub(crate) fn new() -> RenderbufferTarget {
        RenderbufferTarget(RawRenderbufferTarget::new())
    }
}

impl<I: UncompressedFormat> Renderbuffer<I> {
    pub fn new(dims: DimsBox<Point2<u32>>, samples: u32, state: Rc<ContextState>) -> Renderbuffer<I> {
        let mut raw = RawRenderbuffer::new(&state.gl);
        let internal_format = match I::ATTRIBUTES.format {
            GLFormat::Uncompressed{internal_format, ..} => internal_format,
            GLFormat::Compressed{..} => panic!("compressed format information passed with uncompressed texture;\
                                                check the image format's ATTRIBUTES.format field. It should have a\
                                                GLFormat::Uncompressed value")
        };

        unsafe {
            let mut bind = state.renderbuffer_target.0.bind_mut(&mut raw, &state.gl);
            bind.alloc_storage(internal_format, dims, samples);
        }

        Renderbuffer {
            raw, samples, dims, state,
            _format: PhantomData
        }
    }

    #[inline(always)]
    pub fn dims(&self) -> DimsBox<Point2<u32>> {
        self.dims
    }

    #[inline(always)]
    pub fn samples(&self) -> u32 {
        self.samples
    }
}

impl<I: UncompressedFormat> GLObject for Renderbuffer<I> {
    #[inline(always)]
    fn handle(&self) -> Handle {
        self.raw.handle()
    }
}

impl<I: UncompressedFormat> Drop for Renderbuffer<I> {
    fn drop(&mut self) {
        let mut buffer = unsafe{ mem::uninitialized() };
        mem::swap(&mut buffer, &mut self.raw);
        buffer.delete(&self.state);
    }
}