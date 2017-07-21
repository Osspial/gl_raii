use gl;
use gl::types::*;

use {GLScalar, GLSLTypeTransparent, GLSLTypeUniform, GLSLTypeTag, GLSLBasicTag};

use cgmath::{
    Vector1, Vector2, Vector3, Vector4, Point1, Point2, Point3, Matrix2, Matrix3, Matrix4
};

use std::mem;
use std::fmt::{self, Display, Formatter};

macro_rules! impl_glsl_vector {
    ($(impl $vector:ident $num:expr;)*) => {$(
        unsafe impl<P: GLScalar> GLSLTypeTransparent for $vector<P> {
            type Scalar = P;
            #[inline]
            fn prim_tag() -> GLSLBasicTag {Self::Scalar::prim_tag().vectorize($num).unwrap()}
        }
    )*}
}
macro_rules! impl_glsl_matrix {
    ($(impl $matrix:ident $num:expr;)*) => {$(
        // We aren't implementing matrix for normalized integers because that complicates uniform
        // upload. No idea if OpenGL actually supports it either.
        unsafe impl GLSLTypeTransparent for $matrix<f32> {
            type Scalar = f32;
            #[inline]
            fn prim_tag() -> GLSLBasicTag {Self::Scalar::prim_tag().matricize($num, $num).unwrap()}
        }
    )*}
}
// I'm not implementing arrays right now because that's kinda complicated and I'm not convinced
// it's worth the effort rn.
// macro_rules! impl_glsl_array {
//     ($($num:expr),*) => {$(
//         unsafe impl<T: GLSLTypeTransparent> GLSLTypeTransparent for [T; $num] {
//             #[inline]
//             fn len() -> usize {$num}
//             #[inline]
//             fn matrix() -> bool {false}
//             type GLScalar = T::GLScalar;
//         }
//     )*}
// }
// impl_glsl_array!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
//     24, 25, 26, 27, 28, 29, 30, 31, 32);

impl_glsl_vector!{
    impl Vector1 1;
    impl Vector2 2;
    impl Vector3 3;
    impl Vector4 4;
    impl Point1 1;
    impl Point2 2;
    impl Point3 3;
}
impl_glsl_matrix!{
    impl Matrix2 2;
    impl Matrix3 3;
    impl Matrix4 4;
}

macro_rules! impl_gl_scalar_nonorm {
    ($(impl $scalar:ty = ($gl_enum:expr, $prim_tag:ident);)*) => {$(
        unsafe impl GLScalar for $scalar {
            #[inline]
            fn gl_enum() -> GLenum {$gl_enum}
            #[inline]
            fn normalized() -> bool {false}
        }

        unsafe impl GLSLTypeTransparent for $scalar {
            type Scalar = $scalar;
            #[inline]
            fn prim_tag() -> GLSLBasicTag {GLSLBasicTag::$prim_tag}
        }
    )*}
}

impl_gl_scalar_nonorm!{
    impl bool = (gl::BOOL, Bool);
    impl u8 = (gl::UNSIGNED_BYTE, UInt);
    impl u16 = (gl::UNSIGNED_SHORT, UInt);
    impl u32 = (gl::UNSIGNED_INT, UInt);
    impl i8 = (gl::BYTE, Int);
    impl i16 = (gl::SHORT, Int);
    impl i32 = (gl::INT, Int);
    impl f32 = (gl::FLOAT, Float);
    // impl f64 = (gl::DOUBLE, Double);
}

macro_rules! impl_glsl_type_uniform_single {
    ($($ty:ty,)*) => ($(
        unsafe impl GLSLTypeUniform for $ty {
            #[inline]
            fn uniform_tag() -> GLSLTypeTag {
                GLSLTypeTag::Single(Self::prim_tag())
            }
        }
    )*)
}

impl_glsl_type_uniform_single!{
    i32, u32, bool, f32,
    Point1<i32>, Vector1<i32>,
    Point2<i32>, Vector2<i32>,
    Point3<i32>, Vector3<i32>,
    Vector4<i32>,

    Point1<u32>, Vector1<u32>,
    Point2<u32>, Vector2<u32>,
    Point3<u32>, Vector3<u32>,
    Vector4<u32>,

    Point1<bool>, Vector1<bool>,
    Point2<bool>, Vector2<bool>,
    Point3<bool>, Vector3<bool>,
    Vector4<bool>,

    Point1<f32>, Vector1<f32>,
    Point2<f32>, Vector2<f32>,
    Point3<f32>, Vector3<f32>,
    Vector4<f32>,
    Matrix2<f32>,
    Matrix3<f32>,
    Matrix4<f32>,

    // Only supported on on OpenGL 4
    // Point1<f64>, Vector1<f64>,
    // Point2<f64>, Vector2<f64>,
    // Point3<f64>, Vector3<f64>,
    // Vector4<f64>,
    // Matrix2<f64>,
    // Matrix3<f64>,
    // Matrix4<f64>,
}

impl From<GLSLBasicTag> for GLenum {
    fn from(tag: GLSLBasicTag) -> GLenum {
        unsafe{ mem::transmute(tag) }
    }
}

impl Display for GLSLTypeTag {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use self::GLSLTypeTag::*;
        match *self {
            Single(tag) => tag.fmt(f),
            Array(tag, len) => write!(f, "{}[{}]", tag, len)
        }
    }
}

impl Display for GLSLBasicTag {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use self::GLSLBasicTag::*;
        let string = match *self {
            Float => "float",
            Vec2 => "vec2",
            Vec3 => "vec3",
            Vec4 => "vec4",
            // Double => "double",
            // Dvec2 => "dvec2",
            // Dvec3 => "dvec3",
            // Dvec4 => "dvec4",
            Int => "int",
            IVec2 => "ivec2",
            IVec3 => "ivec3",
            IVec4 => "ivec4",
            UInt => "unsigned int",
            UVec2 => "uvec2",
            UVec3 => "uvec3",
            UVec4 => "uvec4",
            Bool => "bool",
            BVec2 => "bvec2",
            BVec3 => "bvec3",
            BVec4 => "bvec4",
            Mat2 => "mat2",
            Mat3 => "mat3",
            Mat4 => "mat4",
            // Mat2x3 => "mat2x3",
            // Mat2x4 => "mat2x4",
            // Mat3x2 => "mat3x2",
            // Mat3x4 => "mat3x4",
            // Mat4x2 => "mat4x2",
            // Mat4x3 => "mat4x3",
            // DMat2 => "dmat2",
            // DMat3 => "dmat3",
            // DMat4 => "dmat4",
            // DMat2x3 => "dmat2x3",
            // DMat2x4 => "dmat2x4",
            // DMat3x2 => "dmat3x2",
            // DMat3x4 => "dmat3x4",
            // DMat4x2 => "dmat4x2",
            // DMat4x3 => "dmat4x3",
            // Sampler1D => "sampler1D",
            // Sampler2D => "sampler2D",
            // Sampler3D => "sampler3D",
            // SamplerCube => "samplerCube",
            // Sampler1DShadow => "sampler1DShadow",
            // Sampler2DShadow => "sampler2DShadow",
            // Sampler1DArray => "sampler1DArray",
            // Sampler2DArray => "sampler2DArray",
            // Sampler1DArrayShadow => "sampler1DArrayShadow",
            // Sampler2DArrayShadow => "sampler2DArrayShadow",
            // Sampler2DMS => "sampler2DMS",
            // Sampler2DMSArray => "sampler2DMSArray",
            // SamplerCubeShadow => "samplerCubeShadow",
            // SamplerBuffer => "samplerBuffer",
            // Sampler2DRect => "sampler2DRect",
            // Sampler2DRectShadow => "sampler2DRectShadow",
            // ISampler1D => "isampler1D",
            // ISampler2D => "isampler2D",
            // ISampler3D => "isampler3D",
            // ISamplerCube => "isamplerCube",
            // ISampler1DArray => "isampler1DArray",
            // ISampler2DArray => "isampler2DArray",
            // ISampler2DMS => "isampler2DMS",
            // ISampler2DMSArray => "isampler2DMSArray",
            // ISamplerBuffer => "isamplerBuffer",
            // ISampler2DRect => "isampler2DRect",
            // USampler1D => "usampler1D",
            // USampler2D => "usampler2D",
            // USampler3D => "usampler3D",
            // USamplerCube => "usamplerCube",
            // USampler1DArray => "usampler1DArray",
            // USampler2DArray => "usampler2DArray",
            // USampler2DMS => "usampler2DMS",
            // USampler2DMSArray => "usampler2DMSArray",
            // USamplerBuffer => "usamplerBuffer",
            // USampler2DRect => "usampler2DRect",
        };

        write!(f, "{}", string)
    }
}

impl GLSLBasicTag {
    pub fn len(self) -> usize {
        use GLSLBasicTag::*;
        match self {
            // Double |
            Int   |
            Float |
            UInt  |
            Bool => 1,

            // Dvec2 |
            Vec2  |
            IVec2 |
            UVec2 |
            BVec2 => 2,

            // Dvec3 |
            Vec3  |
            IVec3 |
            UVec3 |
            BVec3 => 3,

            // Dvec4 |
            Vec4  |
            IVec4 |
            UVec4 |
            BVec4 => 4,

            // DMat2 |
            Mat2 => 4,
            // DMat3 |
            Mat3 => 9,
            // DMat4 |
            Mat4 => 16,
            // DMat2x3 |
            // DMat3x2 |
            // Mat3x2  |
            // Mat2x3 => 6,
            // DMat2x4 |
            // DMat4x2 |
            // Mat4x2  |
            // Mat2x4 => 8,
            // DMat3x4 |
            // DMat4x3 |
            // Mat3x4  |
            // Mat4x3 => 12,
            // Sampler1D |
            // Sampler2D |
            // Sampler3D |
            // SamplerCube |
            // Sampler1DShadow |
            // Sampler2DShadow |
            // Sampler1DArray |
            // Sampler2DArray |
            // Sampler1DArrayShadow |
            // Sampler2DArrayShadow |
            // Sampler2DMS |
            // Sampler2DMSArray |
            // SamplerCubeShadow |
            // SamplerBuffer |
            // Sampler2DRect |
            // Sampler2DRectShadow |
            // ISampler1D |
            // ISampler2D |
            // ISampler3D |
            // ISamplerCube |
            // ISampler1DArray |
            // ISampler2DArray |
            // ISampler2DMS |
            // ISampler2DMSArray |
            // ISamplerBuffer |
            // ISampler2DRect |
            // USampler1D |
            // USampler2D |
            // USampler3D |
            // USamplerCube |
            // USampler1DArray |
            // USampler2DArray |
            // USampler2DMS |
            // USampler2DMSArray |
            // USamplerBuffer |
            // USampler2DRect => 1
        }
    }

    pub fn vectorize(self, len: u8) -> Option<GLSLBasicTag> {
        use GLSLBasicTag::*;
        match (self, len) {
            (Int, 1) => Some(Int),
            (Int, 2) => Some(IVec2),
            (Int, 3) => Some(IVec3),
            (Int, 4) => Some(IVec4),

            (Float, 1) => Some(Float),
            (Float, 2) => Some(Vec2),
            (Float, 3) => Some(Vec3),
            (Float, 4) => Some(Vec4),

            (UInt, 1) => Some(UInt),
            (UInt, 2) => Some(UVec2),
            (UInt, 3) => Some(UVec3),
            (UInt, 4) => Some(UVec4),

            (Bool, 1) => Some(Bool),
            (Bool, 2) => Some(BVec2),
            (Bool, 3) => Some(BVec3),
            (Bool, 4) => Some(BVec4),

            // (Double, 1) => Some(DVec1),
            // (Double, 2) => Some(DVec2),
            // (Double, 3) => Some(DVec3),
            // (Double, 4) => Some(DVec4),
            _ => None
        }
    }

    pub fn matricize(self, width: u8, height: u8) -> Option<GLSLBasicTag> {
        use GLSLBasicTag::*;
        match (self, width, height) {
            (Float, 2, 2) => Some(Mat2),
            (Float, 3, 3) => Some(Mat3),
            (Float, 4, 4) => Some(Mat4),
            // (Float, 2, 3) => Some(Mat2x3),
            // (Float, 2, 4) => Some(Mat2x4),
            // (Float, 3, 2) => Some(Mat3x2),
            // (Float, 3, 4) => Some(Mat3x4),
            // (Float, 4, 2) => Some(Mat4x2),
            // (Float, 4, 3) => Some(Mat4x3),
            // (Double, 2, 2) => Some(DMat2),
            // (Double, 3, 3) => Some(DMat3),
            // (Double, 4, 4) => Some(DMat4),
            // (Double, 2, 3) => Some(DMat2x3),
            // (Double, 2, 4) => Some(DMat2x4),
            // (Double, 3, 2) => Some(DMat3x2),
            // (Double, 3, 4) => Some(DMat3x4),
            // (Double, 4, 2) => Some(DMat4x2),
            // (Double, 4, 3) => Some(DMat4x3),
            _ => None
        }
    }
}