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

use gl;
use gl::types::*;

use glsl::*;
use seal::Sealed;

use cgmath::{Vector1, Vector2, Vector3, Vector4};

use std::slice;

pub unsafe trait ImageFormat: 'static + Copy {
    type Scalar: ScalarNum;

    const INTERNAL_FORMAT: GLenum;
    const PIXEL_FORMAT: GLenum;
    const PIXEL_TYPE: GLenum;
}

pub unsafe trait ColorFormat: ImageFormat {}
pub unsafe trait DepthFormat: ImageFormat {}
pub unsafe trait StencilFormat: ImageFormat {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Depth16(pub u16);
// #[repr(C)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct Depth24(pub u32);
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Depth32F(pub f32);
// #[repr(C)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct Depth24Stencil8(pub u32);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba<S: ScalarNum> {
    pub r: S,
    pub g: S,
    pub b: S,
    pub a: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb<S: ScalarNum> {
    pub r: S,
    pub g: S,
    pub b: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rg<S: ScalarNum> {
    pub r: S,
    pub g: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Red<S: ScalarNum> {
    pub r: S
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

macro_rules! impl_color {
    ($(impl $name:ident<S>($len:expr, colors: $($channel:ident),+);)*) => {$(
        impl<S: ScalarNum> $name<S> {
            impl_color!{impl body $name<S>($len, colors: $($channel),+)}
        }
    )*};
    (impl body $name:ident<$ty:ty>($len:expr, colors: $($channel:ident),+)) => {
        #[inline]
        pub fn new($($channel: $ty),*) -> Self {
            $name{ $($channel),* }
        }

        #[inline(always)]
        pub fn slice_from_raw(raw: &[$ty]) -> &[Self] {
            assert_eq!(0, raw.len() % $len);
            unsafe{ slice::from_raw_parts(raw.as_ptr() as *const Self, raw.len() / $len) }
        }

        #[inline(always)]
        pub fn slice_from_raw_mut(raw: &mut [$ty]) -> &mut [Self] {
            assert_eq!(0, raw.len() % $len);
            unsafe{ slice::from_raw_parts_mut(raw.as_mut_ptr() as *mut Self, raw.len() / $len) }
        }

        #[inline(always)]
        pub fn to_raw_slice(slice: &[Self]) -> &[$ty] {
            unsafe{ slice::from_raw_parts(slice.as_ptr() as *const $ty, slice.len() * $len) }
        }

        #[inline(always)]
        pub fn to_raw_slice_mut(slice: &mut [Self]) -> &mut [$ty] {
            unsafe{ slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut $ty, slice.len() * $len) }
        }
    };
}

impl_color!{
    impl Rgba<S>(4, colors: r, g, b, a);
    impl Rgb<S>(3, colors: r, g, b);
    impl Rg<S>(2, colors: r, g);
    impl Red<S>(1, colors: r);
}

impl SRgba {
    impl_color!{impl body SRgba<u8>(4, colors: r, g, b, a)}
}

impl SRgb {
    impl_color!{impl body SRgb<u8>(3, colors: r, g, b)}
}

impl<S: ScalarNum> Sealed for Rgba<S> {}
impl<S: ScalarNum> Sealed for Rgb<S> {}
impl<S: ScalarNum> Sealed for Rg<S> {}
impl<S: ScalarNum> Sealed for Red<S> {}
impl Sealed for SRgba {}
impl Sealed for SRgb {}

impl<S: ScalarNum> From<Rgb<S>> for Rgba<S> {
    #[inline]
    fn from(colors: Rgb<S>) -> Rgba<S> {
        Rgba::new(colors.r, colors.g, colors.b, S::one())
    }
}
impl<S: ScalarNum> From<Rg<S>> for Rgba<S> {
    #[inline]
    fn from(colors: Rg<S>) -> Rgba<S> {
        Rgba::new(colors.r, colors.g, S::zero(), S::one())
    }
}
impl<S: ScalarNum> From<Red<S>> for Rgba<S> {
    #[inline]
    fn from(colors: Red<S>) -> Rgba<S> {
        Rgba::new(colors.r, S::zero(), S::zero(), S::one())
    }
}

unsafe impl<S: ScalarNum> TypeTransparent for Rgba<S> {
    type Scalar = S;
    const PRIM_TAG: TypeBasicTag = unsafe{ vectorize!(;const; Self::Scalar::PRIM_TAG, 4) };
}
unsafe impl<S: ScalarNum> TypeTransparent for Rgb<S> {
    type Scalar = S;
    const PRIM_TAG: TypeBasicTag = unsafe{ vectorize!(;const; Self::Scalar::PRIM_TAG, 3) };
}
unsafe impl<S: ScalarNum> TypeTransparent for Rg<S> {
    type Scalar = S;
    const PRIM_TAG: TypeBasicTag = unsafe{ vectorize!(;const; Self::Scalar::PRIM_TAG, 2) };
}
unsafe impl<S: ScalarNum> TypeTransparent for Red<S> {
    type Scalar = S;
    const PRIM_TAG: TypeBasicTag = unsafe{ vectorize!(;const; Self::Scalar::PRIM_TAG, 1) };
}
impl<S: ScalarNum> Into<Vector4<S>> for Rgba<S> {
    #[inline]
    fn into(self: Rgba<S>) -> Vector4<S> {
        Vector4::new(self.r, self.g, self.b, self.a)
    }
}
impl<S: ScalarNum> Into<Vector3<S>> for Rgb<S> {
    #[inline]
    fn into(self: Rgb<S>) -> Vector3<S> {
        Vector3::new(self.r, self.g, self.b)
    }
}
impl<S: ScalarNum> Into<Vector2<S>> for Rg<S> {
    #[inline]
    fn into(self: Rg<S>) -> Vector2<S> {
        Vector2::new(self.r, self.g)
    }
}
impl<S: ScalarNum> Into<Vector1<S>> for Red<S> {
    #[inline]
    fn into(self: Red<S>) -> Vector1<S> {
        Vector1::new(self.r)
    }
}

macro_rules! if_or_else {
    (if $if:expr => ($t:expr) else ($f:expr)) => {{
        ($if as GLenum * $t) + ((!$if) as GLenum * $f)
    }};
}

macro_rules! basic_format {
    ($(
        $prim:ty = ($rgba_enum:ident, $rgb_enum:ident, $rg_enum:ident, $r_enum:ident);)
    *) => {$(
        unsafe impl ColorFormat for Rgba<$prim> {}
        unsafe impl ImageFormat for Rgba<$prim> {
            type Scalar = $prim;
            #[inline]
            const INTERNAL_FORMAT: GLenum = gl::$rgba_enum;
            const PIXEL_FORMAT: GLenum = if_or_else!(if <$prim as Scalar>::INTEGER => (gl::RGBA_INTEGER) else (gl::RGBA));
            const PIXEL_TYPE: GLenum = <$prim as Scalar>::GL_ENUM;
        }
        unsafe impl ColorFormat for Rgb<$prim> {}
        unsafe impl ImageFormat for Rgb<$prim> {
            type Scalar = $prim;
            #[inline]
            const INTERNAL_FORMAT: GLenum = gl::$rgb_enum;
            const PIXEL_FORMAT: GLenum = if_or_else!(if <$prim as Scalar>::INTEGER => (gl::RGB_INTEGER) else (gl::RGB));
            const PIXEL_TYPE: GLenum = <$prim as Scalar>::GL_ENUM;
        }
        unsafe impl ColorFormat for Rg<$prim> {}
        unsafe impl ImageFormat for Rg<$prim> {
            type Scalar = $prim;
            #[inline]
            const INTERNAL_FORMAT: GLenum = gl::$rg_enum;
            const PIXEL_FORMAT: GLenum = if_or_else!(if <$prim as Scalar>::INTEGER => (gl::RG_INTEGER) else (gl::RG));
            const PIXEL_TYPE: GLenum = <$prim as Scalar>::GL_ENUM;
        }
        unsafe impl ColorFormat for Red<$prim> {}
        unsafe impl ImageFormat for Red<$prim> {
            type Scalar = $prim;
            #[inline]
            const INTERNAL_FORMAT: GLenum = gl::$r_enum;
            const PIXEL_FORMAT: GLenum = if_or_else!(if <$prim as Scalar>::INTEGER => (gl::RED_INTEGER) else (gl::RED));
            const PIXEL_TYPE: GLenum = <$prim as Scalar>::GL_ENUM;
        }
    )*}
}

basic_format!{
    u8 = (RGBA8, RGB8, RG8, R8);
    u16 = (RGBA16, RGB16, RG16, R16);

    i8 = (RGBA8_SNORM, RGB8_SNORM, RG8_SNORM, R8_SNORM);
    i16 = (RGBA16_SNORM, RGB16_SNORM, RG16_SNORM, R16_SNORM);

    f32 = (RGBA32F, RGB32F, RG32F, R32F);

    GLSLInt<u8> = (RGBA8UI, RGB8UI, RG8UI, R8UI);
    GLSLInt<u16> = (RGBA16UI, RGB16UI, RG16UI, R16UI);
    GLSLInt<u32> = (RGBA32UI, RGB32UI, RG32UI, R32UI);

    GLSLInt<i8> = (RGBA8I, RGB8I, RG8I, R8I);
    GLSLInt<i16> = (RGBA16I, RGB16I, RG16I, R16I);
    GLSLInt<i32> = (RGBA32I, RGB32I, RG32I, R32I);
}
unsafe impl ColorFormat for SRgba {}
unsafe impl ImageFormat for SRgba {
    type Scalar = u8;
    const INTERNAL_FORMAT: GLenum =  gl::SRGB8_ALPHA8 ;
    const PIXEL_FORMAT: GLenum =  gl::RGBA;
    const PIXEL_TYPE: GLenum = <u8 as Scalar>::GL_ENUM;
}
unsafe impl ColorFormat for SRgb {}
unsafe impl ImageFormat for SRgb {
    type Scalar = u8;
    const INTERNAL_FORMAT: GLenum =  gl::SRGB8 ;
    const PIXEL_FORMAT: GLenum =  gl::RGB;
    const PIXEL_TYPE: GLenum = <u8 as Scalar>::GL_ENUM;
}

// unsafe impl ImageFormat for Depth16 {
//     type Scalar =
// }
