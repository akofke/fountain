use std::sync::Arc;
use crate::material::Material;
use crate::Transform;
use crate::light::diffuse::DiffuseAreaLightBuilder;
use pbrt_parser as parser;
use pbrt_parser::WorldStmt;
use crate::loaders::{ParamSet, ParamVal};
use crate::spectrum::Spectrum;

struct PbrtSceneBuilder {
    graphics_state: Vec<GraphicsState>,
    tf_state: Vec<Transform>,
}

struct GraphicsState {
    material: Arc<dyn Material>,
    area_light: DiffuseAreaLightBuilder,
}

impl PbrtSceneBuilder {

    // TODO: convert in place!
    fn convert_param_val(&self, val: parser::ParamVal) -> ParamVal {
        match val {
            parser::ParamVal::Int(v) => ParamVal::Int(v.into()),
            parser::ParamVal::Float(v) => ParamVal::Float(v.into()),
            parser::ParamVal::Point2(v) => ParamVal::Point2f(convert_vec(v).into()),
            parser::ParamVal::Point3(v) => ParamVal::Point3f(convert_vec(v).into()),
            parser::ParamVal::Vector2(v) => ParamVal::Vec2f(convert_vec(v).into()),
            parser::ParamVal::Vector3(v) => ParamVal::Vec3f(convert_vec(v).into()),
            parser::ParamVal::Normal3(v) => ParamVal::Normal3(convert_vec(v).into()),
            parser::ParamVal::Bool(v) => ParamVal::Bool(v.into()),
            parser::ParamVal::String(v) => ParamVal::String(v.into_iter().map(|s| s.to_string()).collect::<Vec<_>>().into()),
            parser::ParamVal::Texture(s) => unimplemented!(),
            parser::ParamVal::SpectrumRgb(v) => {
                ParamVal::Spectrum(v.into_iter().map(|s| s.into()).collect::<Vec<Spectrum>>().into())
            },
            parser::ParamVal::SpectrumXyz(_) => unimplemented!(),
            parser::ParamVal::SpectrumSampled(_) => unimplemented!(),
            parser::ParamVal::SpectrumBlackbody(_) => unimplemented!(),
        }
    }

    fn make_param_set(&self, params: Vec<parser::Param>) -> ParamSet {
        unimplemented!()
    }

    fn exec_stmt(&mut self, stmt: parser::WorldStmt) {
        match stmt {
            WorldStmt::AttributeBegin => {},
            WorldStmt::AttributeEnd => {},
            WorldStmt::TransformBegin => {},
            WorldStmt::TransformEnd => {},
            WorldStmt::ObjectBegin(_) => {},
            WorldStmt::ObjectEnd => {},
            WorldStmt::ReverseOrientation => {},
            WorldStmt::Transform(_) => {},
            WorldStmt::Shape(_, _) => {},
            WorldStmt::ObjectInstance(_) => {},
            WorldStmt::LightSource(_, _) => {},
            WorldStmt::AreaLightSource(_, _) => {},
            WorldStmt::Material(_, _) => {},
            WorldStmt::MakeNamedMaterial(_, _) => {},
            WorldStmt::NamedMaterial(_) => {},
            WorldStmt::Texture(_) => {},
            WorldStmt::MakeNamedMedium(_, _) => {},
            WorldStmt::MediumInterface(_, _) => {},
            WorldStmt::Include(_) => {},
        }
    }

    fn shape(&mut self, name: Arc<str>, params: ParamSet) -> Result<(), ()> {
        match name.as_ref() {
            "sphere" => {
                unimplemented!()
            },
            _ => {
                Err(())
            }
        }
    }
}



fn convert_vec<T, U: From<T>>(v: Vec<T>) -> Vec<U> {
    v.into_iter().map(Into::into).collect()
}