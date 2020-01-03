use std::sync::Arc;
use crate::material::Material;
use crate::{Transform, Point3f, Vec3f};
use crate::Float;
use crate::light::diffuse::DiffuseAreaLightBuilder;
use pbrt_parser as parser;
use pbrt_parser::{WorldStmt, TransformStmt};
use crate::loaders::{ParamSet, ParamVal, ParamError};
use crate::spectrum::Spectrum;
use std::collections::HashMap;
use crate::texture::Texture;
use crate::loaders::constructors::make_sphere;

struct PbrtSceneBuilder {
    graphics_state: Vec<GraphicsState>,
    tf_state: Vec<Transform>,
    float_textures: HashMap<String, Arc<dyn Texture<Output=Float>>>,
    spectrum_textures: HashMap<String, Arc<dyn Texture<Output=Spectrum>>>,
}

struct GraphicsState {
    material: Arc<dyn Material>,
    area_light: DiffuseAreaLightBuilder,
}

pub enum PbrtEvalError {
    ParamError(ParamError),
    TextureError {
        expected: String
    },
    UnknownName(String),
}

impl From<ParamError> for PbrtEvalError {
    fn from(e: ParamError) -> Self {
        Self::ParamError(e)
    }
}

impl PbrtSceneBuilder {

    // TODO: convert in place!
    fn convert_param_val(&self, val: parser::ParamVal) -> Result<ParamVal, PbrtEvalError> {
        let value = match val {
            parser::ParamVal::Int(v) => ParamVal::Int(v.into()),
            parser::ParamVal::Float(v) => ParamVal::Float(v.into()),
            parser::ParamVal::Point2(v) => ParamVal::Point2f(convert_vec(v).into()),
            parser::ParamVal::Point3(v) => ParamVal::Point3f(convert_vec(v).into()),
            parser::ParamVal::Vector2(v) => ParamVal::Vec2f(convert_vec(v).into()),
            parser::ParamVal::Vector3(v) => ParamVal::Vec3f(convert_vec(v).into()),
            parser::ParamVal::Normal3(v) => ParamVal::Normal3(convert_vec(v).into()),
            parser::ParamVal::Bool(v) => ParamVal::Bool(v.into()),
            parser::ParamVal::String(v) => ParamVal::String(v.into_iter().map(|s| s.to_string()).collect::<Vec<_>>().into()),
            parser::ParamVal::Texture(s) => self.lookup_texture(&s[0])?, // TODO: no vec for textures
            parser::ParamVal::SpectrumRgb(v) => {
                ParamVal::Spectrum(v.into_iter().map(|s| s.into()).collect::<Vec<Spectrum>>().into())
            },
            parser::ParamVal::SpectrumXyz(_) => unimplemented!(),
            parser::ParamVal::SpectrumSampled(_) => unimplemented!(),
            parser::ParamVal::SpectrumBlackbody(_) => unimplemented!(),
        };
        Ok(value)
    }

    fn lookup_texture(&self, name: &str) -> Result<ParamVal, PbrtEvalError> {
        self.spectrum_textures.get(name)
            .map(|t| ParamVal::from(t.clone()))
            .or_else(|| self.float_textures.get(name).map(|t| ParamVal::from(t.clone())))
            .ok_or_else(|| PbrtEvalError::TextureError { expected: name.to_string() })
    }

    fn make_param_set(&self, params: Vec<parser::Param>) -> Result<ParamSet, PbrtEvalError> {
        let mut map = params.into_iter().map(|param| {
            let name = param.name.to_string();
            let val = self.convert_param_val(param.value)?;
            Ok((name, val))
        }).collect::<Result<HashMap<String, ParamVal>, PbrtEvalError>>()?;
        Ok(ParamSet { params: map })
    }

    fn current_tf_mut(&mut self) -> &mut Transform {
        self.tf_state.last_mut().expect("Transform stack empty")
    }

    fn exec_stmt(&mut self, stmt: parser::WorldStmt) -> Result<(), PbrtEvalError> {
        match stmt {
            WorldStmt::AttributeBegin => {},
            WorldStmt::AttributeEnd => {},
            WorldStmt::TransformBegin => {},
            WorldStmt::TransformEnd => {},
            WorldStmt::ObjectBegin(_) => {},
            WorldStmt::ObjectEnd => {},
            WorldStmt::ReverseOrientation => {},
            WorldStmt::Transform(tf_stmt) => {
                let ctm = self.tf_state.pop().unwrap();
                let ctm = eval_transform_stmt(tf_stmt, &ctm)?;
                self.tf_state.push(ctm)
            },
            WorldStmt::Shape(name, params) => {

            },
            WorldStmt::ObjectInstance(_) => {},
            WorldStmt::LightSource(_, _) => {},
            WorldStmt::AreaLightSource(_, _) => {},
            WorldStmt::Material(name, params) => {

            },
            WorldStmt::MakeNamedMaterial(_, _) => {},
            WorldStmt::NamedMaterial(_) => {},
            WorldStmt::Texture(_) => {},
            WorldStmt::MakeNamedMedium(_, _) => {},
            WorldStmt::MediumInterface(_, _) => {},
            WorldStmt::Include(_) => {},
        };
        unimplemented!()
    }

    fn shape(&mut self, name: Arc<str>, params: ParamSet) -> Result<(), PbrtEvalError> {
        match name.as_ref() {
            "sphere" => {
                let shape = make_sphere(params)?;
                unimplemented!()
            },
            _ => {
                Err(PbrtEvalError::UnknownName(name.to_string()))
            }
        }
    }

    fn material(&mut self, name: Arc<str>, params: ParamSet) -> Result<(), PbrtEvalError> {
        unimplemented!()
    }
}

fn eval_transform_stmt(stmt: parser::TransformStmt, current_tf: &Transform) -> Result<Transform, PbrtEvalError> {
    let tf = match stmt {
        parser::TransformStmt::Identity => {
            Transform::identity()
        },
        parser::TransformStmt::Translate(v) => {
            *current_tf * Transform::translate((*v).into())
        },
        parser::TransformStmt::Scale(v) => {
            *current_tf * Transform::scale(v[0], v[1], v[2])
        },
        parser::TransformStmt::Rotate(v) => {
            unimplemented!()
        },
        parser::TransformStmt::LookAt(m) => {
            let eye = Point3f::new(m[0], m[1], m[2]);
            let look_at = Point3f::new(m[3], m[4], m[5]);
            let up = Vec3f::new(m[6], m[7], m[8]);
            *current_tf * Transform::look_at(eye, look_at, up)
        },
        parser::TransformStmt::CoordinateSystem(_) => {
            unimplemented!()
        },
        parser::TransformStmt::CoordSysTransform(_) => {
            unimplemented!()
        },
        parser::TransformStmt::Transform(m) => {
            unimplemented!()
        },
        parser::TransformStmt::ConcatTransform(m) => {
            unimplemented!()
        },
    };
    Ok(tf)
}



fn convert_vec<T, U: From<T>>(v: Vec<T>) -> Vec<U> {
    v.into_iter().map(Into::into).collect()
}