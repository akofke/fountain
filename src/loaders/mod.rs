use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use smallvec::SmallVec;
use crate::{Point2f, Vec2f, Vec3f, Point3f, Normal3, Float, Transform};
use crate::spectrum::Spectrum;
use std::sync::Arc;
use crate::texture::{Texture, ConstantTexture};
use crate::material::Material;
use std::any::type_name;

pub mod pbrt;
pub mod constructors;

pub enum ParamVal {
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

pub struct TryFromParamErr<V>(&'static str, V);

#[derive(Debug)]
pub struct ParamError {
    pub expected_ty: &'static str,
    pub expected_name: &'static str,
}

impl TryFrom<ParamVal> for Transform {
    type Error = TryFromParamErr<ParamVal>;

    fn try_from(value: ParamVal) -> Result<Self, Self::Error> {
        match value {
            ParamVal::Transform(tf) => Ok(tf),
            _ => Err(TryFromParamErr("transform", value)),
        }
    }
}

impl From<Transform> for ParamVal {
    fn from(v: Transform) -> Self {
        Self::Transform(v)
    }
}

impl TryFrom<ParamVal> for Arc<dyn Texture<Output=Float>> {
    type Error = TryFromParamErr<ParamVal>;

    fn try_from(value: ParamVal) -> Result<Self, Self::Error> {
        match value {
            ParamVal::FloatTexture(tex) => Ok(tex),
            _ => Err(TryFromParamErr("float_texture", value)),
        }
    }
}

impl TryFrom<ParamVal> for Arc<dyn Texture<Output=Spectrum>> {
    type Error = TryFromParamErr<ParamVal>;

    fn try_from(value: ParamVal) -> Result<Self, Self::Error> {
        match value {
            ParamVal::SpectrumTexture(tex) => Ok(tex),
            _ => Err(TryFromParamErr("spectrum_texture", value)),
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
            type Error = TryFromParamErr<ParamVal>;

            fn try_from(value: ParamVal) -> Result<Self, Self::Error> {
                match value {
                    ParamVal::$param_variant(v) if v.len() == 1 => {
                        Ok(v.into_iter().nth(0).unwrap())
                    },
                    _ => Err(TryFromParamErr($ty_name, value))
                }
            }
        }

        impl<'a> TryFrom<&'a ParamVal> for &'a $into_ty {
            type Error = TryFromParamErr<&'a ParamVal>;

            fn try_from(value: &'a ParamVal) -> Result<Self, Self::Error> {
                match value {
                    ParamVal::$param_variant(v) if v.len() == 1 => {
                        Ok(&v[0])
                    },
                    _ => Err(TryFromParamErr($ty_name, value))
                }
            }
        }

        impl TryFrom<ParamVal> for Vec<$into_ty> {
            type Error = TryFromParamErr<ParamVal>;

            fn try_from(value: ParamVal) -> Result<Self, Self::Error> {
                match value {
                    ParamVal::$param_variant(v) => {
                        Ok(v.into_vec())
                    },
                    _ => Err(TryFromParamErr($ty_name, value))
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


#[derive(Default)]
pub struct ParamSet {
    params: HashMap<String, ParamVal>,
}

impl ParamSet {
    pub fn new() -> Self {
        Self {
            params: Default::default(),
        }
    }

    pub fn put_one(&mut self, name: String, val: impl Into<ParamVal>) {
        self.params.insert(name, val.into());
    }

    pub fn get_one<T>(&mut self, name: &'static str) -> Result<T, ParamError>
        where T: TryFrom<ParamVal, Error=TryFromParamErr<ParamVal>>
    {
        self.params.remove(name)
            .ok_or_else(|| ParamError { expected_name: name, expected_ty: type_name::<T>()})?
            .try_into()
            .map_err(|e: TryFromParamErr<_>| {
                self.params.insert(name.to_string(), e.1);
                ParamError { expected_name: name, expected_ty: e.0 }
            })

    }

    pub fn get_one_ref<'a, T>(&'a self, name: &'static str) -> Result<&'a T, ParamError>
        where &'a T: TryFrom<&'a ParamVal, Error=TryFromParamErr<&'a ParamVal>>
    {
        let val = self.params.get(name)
            .ok_or_else(|| ParamError { expected_name: name, expected_ty: type_name::<T>()})?;
        let val = val
            .try_into()
            .map_err(|e: TryFromParamErr<_>| ParamError { expected_name: name, expected_ty: e.0 });
        val

    }

    pub fn get_many<T>(&mut self, name: &'static str) -> Result<Vec<T>, ParamError>
        where Vec<T>: TryFrom<ParamVal, Error=TryFromParamErr<ParamVal>>
    {
        self.get_one::<Vec<T>>(name)
    }

    pub fn get_constant_texture<T: 'static>(&mut self, name: &'static str) -> Result<Arc<dyn Texture<Output=T>>, ParamError>
        where T: TryFrom<ParamVal, Error=TryFromParamErr<ParamVal>> + Copy + Sync + Send
    {
        dbg!("looking for const texture");
        let val = self.get_one::<T>(name)?;
        dbg!("found val");
        Ok(Arc::new(ConstantTexture(val)))
    }

    pub fn get_texture_or_const<T>(&mut self, name: &'static str) -> Result<Arc<dyn Texture<Output=T>>, ParamError>
        where
            T: TryFrom<ParamVal, Error=TryFromParamErr<ParamVal>> + Copy + Sync + Send + 'static,
            Arc<dyn Texture<Output=T>>: TryFrom<ParamVal, Error=TryFromParamErr<ParamVal>>
    {
        let val = self.params.remove(name).ok_or_else(|| ParamError { expected_name: name, expected_ty: type_name::<T>()})?;
        val.try_into()
            .or_else(|e: TryFromParamErr<ParamVal>| {
                let tex_value: T = e.1.try_into().map_err(|e: TryFromParamErr<ParamVal>| ParamError { expected_name: name, expected_ty: e.0 })?;
                Ok(Arc::new(ConstantTexture(tex_value)) as Arc<dyn Texture<Output=T>>)
            })
//        self.get_one::<Arc<dyn Texture<Output=T>>>(name).or_else(|_| self.get_constant_texture(name))
    }

    pub fn get_texture_or_default<T>(&mut self, name: &'static str, default: T) -> Result<Arc<dyn Texture<Output=T>>, ParamError>
        where
            T: TryFrom<ParamVal, Error=TryFromParamErr<ParamVal>> + Copy + Sync + Send + 'static,
            Arc<dyn Texture<Output=T>>: TryFrom<ParamVal, Error=TryFromParamErr<ParamVal>>
    {
        self.get_texture_or_const(name)
            .or_else(|err| {
                Ok(Arc::new(ConstantTexture(default)))
            })
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