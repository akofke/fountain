use std::collections::HashMap;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use smallvec::SmallVec;
use crate::{Point2f, Vec2f, Vec3f, Point3f, Normal3, Float, Transform};
use crate::spectrum::Spectrum;
use std::sync::Arc;
use crate::texture::Texture;
use crate::material::Material;
use std::any::type_name;

pub mod pbrt;
pub mod constructors;

enum ParamVal {
    Int(SmallVec<[i32; 1]>),
    Float(SmallVec<[Float; 1]>),
    Point2f(SmallVec<[Point2f; 1]>),
    Vec2f(SmallVec<[Vec2f; 1]>),
    Point3f(SmallVec<[Point3f; 1]>),
    Vec3f(SmallVec<[Vec3f; 1]>),
    Normal3(SmallVec<[Normal3; 1]>),
    Spectrum(SmallVec<[Spectrum; 1]>),
    Bool(SmallVec<[bool; 1]>),
    String(SmallVec<[String; 1]>),

    Transform(Transform),
    FloatTexture(Arc<dyn Texture<Output=Float>>),
    SpectrumTexture(Arc<dyn Texture<Output=Spectrum>>),
    Material(Arc<dyn Material>),
}

pub struct TryFromParamErr(&'static str);

pub struct ParamError {
    pub expected_ty: &'static str,
    pub expected_name: &'static str,
}

impl TryFrom<ParamVal> for Transform {
    type Error = TryFromParamErr;

    fn try_from(value: ParamVal) -> Result<Self, Self::Error> {
        match value {
            ParamVal::Transform(tf) => Ok(tf),
            _ => Err(TryFromParamErr("transform")),
        }
    }
}

impl From<Arc<dyn Texture<Output=Float>>> for ParamVal {
    fn from(value: Arc<dyn Texture<Output=Float>>) -> Self {
        Self::FloatTexture(value)
    }
}

impl From<Arc<dyn Texture<Output=Spectrum>>> for ParamVal {
    fn from(value: Arc<dyn Texture<Output=Spectrum>>) -> Self {
        Self::SpectrumTexture(value)
    }
}

macro_rules! impl_basic_conversions {
    ($param_variant:ident, $into_ty:ty, $ty_name:expr) => {
        impl TryFrom<ParamVal> for $into_ty {
            type Error = TryFromParamErr;

            fn try_from(value: ParamVal) -> Result<Self, Self::Error> {
                match value {
                    ParamVal::$param_variant(v) => {
                        if v.len() == 1 {
                            v.into_iter().nth(0).ok_or(TryFromParamErr($ty_name))
                        } else {
                            Err(TryFromParamErr($ty_name))
                        }
                    },
                    _ => Err(TryFromParamErr($ty_name))
                }
            }
        }

        impl TryFrom<ParamVal> for Vec<$into_ty> {
            type Error = TryFromParamErr;

            fn try_from(value: ParamVal) -> Result<Self, Self::Error> {
                match value {
                    ParamVal::$param_variant(v) => {
                        Ok(v.into_vec())
                    },
                    _ => Err(TryFromParamErr($ty_name))
                }
            }
        }

        impl From<Vec<$into_ty>> for ParamVal {
            fn from(value: Vec<$into_ty>) -> ParamVal {
                ParamVal::$param_variant(SmallVec::from_vec(value))
            }
        }
    };
}

impl_basic_conversions!(Int, i32, "int");
impl_basic_conversions!(Float, Float, "float");
impl_basic_conversions!(Point2f, Point2f, "point2");
impl_basic_conversions!(Vec2f, Vec2f, "vector2");
impl_basic_conversions!(Point3f, Point3f, "point3");
impl_basic_conversions!(Vec3f, Vec3f, "vector3");
impl_basic_conversions!(Normal3, Normal3, "normal3");
impl_basic_conversions!(Spectrum, Spectrum, "spectrum");
impl_basic_conversions!(Bool, bool, "bool");
impl_basic_conversions!(String, String, "string");


pub struct ParamSet {
    params: HashMap<String, ParamVal>,

}

impl ParamSet {
    pub fn get_one<T>(&mut self, name: &'static str) -> Result<T, ParamError>
        where T: TryFrom<ParamVal, Error=TryFromParamErr>
    {
        self.params.remove(name)
            .ok_or_else(|| ParamError { expected_name: name, expected_ty: type_name::<T>()})?
            .try_into()
            .map_err(|e: TryFromParamErr| ParamError { expected_name: name, expected_ty: e.0 })

    }

    pub fn get_many<T>(&mut self, name: &'static str) -> Result<Vec<T>, ParamError>
        where Vec<T>: TryFrom<ParamVal, Error=TryFromParamErr>
    {
        self.get_one::<Vec<T>>(name)
    }

    pub fn current_transform(&mut self) -> Result<Transform, ParamError> {
        self.get_one("object_to_world")
    }

    pub fn reverse_orientation(&mut self) -> Result<bool, ParamError> {
        self.get_one("reverse_orientation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}