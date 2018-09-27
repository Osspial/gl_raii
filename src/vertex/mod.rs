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

pub(crate) mod vao;
pub use self::vao::VertexArrayObject;

use glsl::TransparentType;
use std::marker::PhantomData;

pub trait VertexMemberRegistry {
    type Group: Vertex;
    /// Add a member to the registry. Note that the value pointed to by `get_type` is allowed to be
    /// instantiated with `mem::zeroed()`, and any references inside should not be dereferenced.
    fn add_member<T>(&mut self, name: &str, get_type: fn(*const Self::Group) -> *const T)
        where T: TransparentType;
}

pub trait Vertex: 'static + Copy {
    fn members<M>(reg: M)
        where M: VertexMemberRegistry<Group=Self>;

    #[inline]
    fn num_members() -> usize {
        struct MemberCounter<'a, G>(&'a mut usize, PhantomData<G>);
        impl<'a, G: Vertex> VertexMemberRegistry for MemberCounter<'a, G> {
            type Group = G;
            #[inline]
            fn add_member<T>(&mut self, _: &str, _: fn(*const G) -> *const T)
                where T: TransparentType
            {
                *self.0 += 1;
            }
        }

        let mut num = 0;
        Self::members(MemberCounter::<Self>(&mut num, PhantomData));
        num
    }
}
