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
use crate::loaders::constructors::{make_sphere, make_matte, make_triangle_mesh, make_diffuse_area_light, ConstructError, make_checkerboard_spect, make_checkerboard_float, make_point_light, make_distant_light};
use crate::light::{AreaLightBuilder, Light};
use crate::primitive::{GeometricPrimitive, Primitive};
use crate::shapes::triangle::TriangleMesh;

use crate::texture::{SpectrumTexture, FloatTexture};
use crate::scene::Scene;
use crate::bvh::BVH;

pub struct PbrtSceneBuilder {
    graphics_state: Vec<GraphicsState>,
    tf_state: Vec<Transform>,
    float_textures: HashMap<String, Arc<dyn Texture<Output=Float>>>,
    spectrum_textures: HashMap<String, Arc<dyn Texture<Output=Spectrum>>>,
    named_materials: HashMap<String, Arc<dyn Material>>,

    primitives: Vec<Box<dyn Primitive>>,
    meshes: Vec<Arc<TriangleMesh>>,
    lights: Vec<Box<dyn Light>>,
}

#[derive(Clone)]
struct GraphicsState {
    material: Option<Arc<dyn Material>>,
    area_light: Option<DiffuseAreaLightBuilder>,
    rev_orientation: bool,
}

pub enum PbrtEvalError {
    ConstructError(ConstructError),
    TextureError {
        expected: String
    },
    MaterialError {
        expected: String
    },
    UnknownName(String),
}

impl From<ParamError> for PbrtEvalError {
    fn from(e: ParamError) -> Self {
        Self::ConstructError(ConstructError::ParamError(e))
    }
}

impl From<ConstructError> for PbrtEvalError {
    fn from(e: ConstructError) -> Self {
        Self::ConstructError(e)
    }
}

impl PbrtSceneBuilder {

    pub fn create_scene(&mut self) -> Scene {
        let bvh = BVH::build(self.primitives);
        let mut lights = Vec::with_capacity(self.lights.len());
        for light in &mut self.lights {
            lights.push(light.as_mut())
        }
        let scene = Scene::new(bvh, lights);
        scene
    }

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

    fn add_spect_tex(&mut self, name: String, tex: Arc<dyn SpectrumTexture>) {
        self.spectrum_textures.insert(name, tex);
    }

    fn add_float_tex(&mut self, name: String, tex: Arc<dyn FloatTexture>) {
        self.float_textures.insert(name, tex);
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

    fn graphics_state_mut(&mut self) -> &mut GraphicsState {
        self.graphics_state.last_mut().unwrap()
    }

    pub fn exec_stmt(&mut self, stmt: parser::WorldStmt) -> Result<(), PbrtEvalError> {
        match stmt {
            WorldStmt::AttributeBegin => {
                let new_state = self.graphics_state.last().unwrap().clone();
                self.graphics_state.push(new_state);
                let new_tf = self.tf_state.last().unwrap().clone();
                self.tf_state.push(new_tf);
            },
            WorldStmt::AttributeEnd => {
                self.graphics_state.pop().unwrap();
                self.tf_state.pop().unwrap();
            },
            WorldStmt::TransformBegin => {
                self.tf_state.push(self.tf_state.last().unwrap().clone());
            },
            WorldStmt::TransformEnd => {
                self.tf_state.pop().unwrap();
            },
            WorldStmt::ObjectBegin(_) => {
                unimplemented!()
            },
            WorldStmt::ObjectEnd => {
                unimplemented!()
            },
            WorldStmt::ReverseOrientation => {
                self.graphics_state_mut().rev_orientation = true;
            },
            WorldStmt::Transform(tf_stmt) => {
                let ctm = self.tf_state.pop().unwrap();
                let ctm = eval_transform_stmt(tf_stmt, &ctm)?;
                self.tf_state.push(ctm)
            },
            WorldStmt::Shape(name, params) => {
                let params = self.make_param_set(params)?;
                self.shape(name, params)?;
            },
            WorldStmt::ObjectInstance(_) => {},
            WorldStmt::LightSource(name, params) => {
                let params = self.make_param_set(params)?;

            },
            WorldStmt::AreaLightSource(name, params) => {
                let params = self.make_param_set(params)?;
                self.area_light(name, params)?;
            },
            WorldStmt::Material(name, params) => {
                let params = self.make_param_set(params)?;
                let mat = self.material(name.as_ref(), params)?;
                self.set_current_material(mat)
            },
            WorldStmt::MakeNamedMaterial(name, params) => {
                let mut params = self.make_param_set(params)?;
                let mat_type: String = params.get_one("type")?;
                let mat = self.material(mat_type.as_ref(), params)?;
                self.named_materials.insert(name.to_string(), mat);
            },
            WorldStmt::NamedMaterial(name) => {
                let mat = self.named_materials
                    .get(name.as_ref())
                    .ok_or_else(|| PbrtEvalError::MaterialError { expected: name.to_string() })?;
                self.set_current_material(mat.clone())
            },
            WorldStmt::Texture(tex_stmt) => {
                let params = self.make_param_set(tex_stmt.params)?;
                self.texture(&tex_stmt.name, &tex_stmt.ty, &tex_stmt.class, params)?;
            },
            WorldStmt::MakeNamedMedium(_, _) => {
                unimplemented!()
            },
            WorldStmt::MediumInterface(_, _) => {
                unimplemented!()
            },
            WorldStmt::Include(_) => {
                unimplemented!()
            },
        };
        Ok(())
    }

    fn shape(&mut self, name: Arc<str>, params: ParamSet) -> Result<(), PbrtEvalError> {
        let graphics_state = self.graphics_state.last_mut().unwrap();
        match name.as_ref() {
            "sphere" => {
                let shape = make_sphere(params)?;
                let shape = Arc::new(shape);
                let light = graphics_state.area_light.clone()
                    .map(|builder| builder.create(shape.clone()));
                let prim = GeometricPrimitive {
                    shape,
                    material: graphics_state.material.clone(),
                    light
                };
                self.primitives.push(Box::new(prim));
            },

            "trianglemesh" => {
                let mesh = make_triangle_mesh(params)?;
                let mesh = Arc::new(mesh);
                self.meshes.push(mesh.clone());
                self.primitives.extend(mesh.iter_triangles()
                    .map(|shape| {
                        let shape = Arc::new(shape);
                        let light = graphics_state.area_light.clone()
                            .map(|builder| builder.create(shape.clone()));
                        let material = graphics_state.material.clone();
                        let prim = GeometricPrimitive {
                            shape,
                            material,
                            light
                        };
                        Box::new(prim) as Box<dyn Primitive>
                    })
                );
            },

            _ => {
                return Err(PbrtEvalError::UnknownName(name.to_string()));
            }
        };
        Ok(())
    }

    fn material(&mut self, name: &str, params: ParamSet) -> Result<Arc<dyn Material>, PbrtEvalError> {
        let material: Arc<dyn Material> = match name {
            "matte" => {
                Arc::new(make_matte(params)?)
            },
            _ => {
                return Err(PbrtEvalError::UnknownName(name.to_string()))
            }
        };
        Ok(material)
    }

    fn set_current_material(&mut self, mat: Arc<dyn Material>) {
        self.graphics_state.last_mut().unwrap().material = Some(mat)
    }

    fn area_light(&mut self, name: Arc<str>, params: ParamSet) -> Result<(), PbrtEvalError> {
        match name.as_ref() {
            "diffuse" => {
                let builder = make_diffuse_area_light(params)?;
                self.graphics_state_mut().area_light = Some(builder);
                Ok(())
            },
            _ => Err(PbrtEvalError::UnknownName(name.to_string()))
        }
    }

    fn texture(&mut self, name: &str, ty: &str, class: &str, params: ParamSet) -> Result<(), PbrtEvalError> {
        match (ty, class) {
            ("spectrum", "checkerboard") => {
                let tex = make_checkerboard_spect(params)?;
                self.add_spect_tex(name.to_string(), tex);
            },
            ("float", "checkerboard") => {
                let tex = make_checkerboard_float(params)?;
                self.add_float_tex(name.to_string(), tex);
            },
            _ => {
                return Err(PbrtEvalError::UnknownName(format!("{} {}", ty, class)));
            }
        };
        Ok(())
    }

    fn light_source(&mut self, name: &str, params: ParamSet) -> Result<(), PbrtEvalError> {
        match name {
            "point" => {
                let light = make_point_light(params)?;
                self.lights.push(Box::new(light));
            },
            "distant" => {
                let light = make_distant_light(params)?;
                self.lights.push(Box::new(light));
            }
            _ => return Err(PbrtEvalError::UnknownName(name.to_string())),
        };
        Ok(())
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