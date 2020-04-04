use std::sync::Arc;
use crate::material::Material;
use crate::{Transform, Point3f, Vec3f, Point2f, Bounds2f, Point2i};
use crate::Float;
use crate::light::diffuse::DiffuseAreaLightBuilder;
use pbrt_parser as parser;
use pbrt_parser::{WorldStmt, TransformStmt, HeaderStmt};
use crate::loaders::{ParamSet, ParamVal, ParamError};
use crate::spectrum::Spectrum;
use std::collections::HashMap;
use crate::texture::Texture;
use crate::loaders::constructors::{make_sphere, make_matte, make_triangle_mesh, make_diffuse_area_light, ConstructError, make_checkerboard_spect, make_checkerboard_float, make_point_light, make_distant_light, make_imagemap_spect, make_infinite_area_light, make_triangle_mesh_from_ply, make_glass, make_metal_material, make_plastic_material, make_mirror_material, make_uv_spect};
use crate::light::{AreaLightBuilder, Light};
use crate::primitive::{GeometricPrimitive, Primitive};
use crate::shapes::triangle::TriangleMesh;

use crate::texture::{SpectrumTexture, FloatTexture};
use crate::scene::Scene;
use crate::bvh::BVH;
use crate::camera::{Camera, PerspectiveCamera};
use crate::sampler::Sampler;
use crate::filter::BoxFilter;
use crate::sampler::random::RandomSampler;
use crate::film::Film;
use cgmath::Deg;
use std::fmt::{Formatter, Error};

pub struct PbrtSceneBuilder {
    graphics_state: Vec<GraphicsState>,
    tf_state: Vec<Transform>,
    float_textures: HashMap<String, Arc<dyn Texture<Output=Float>>>,
    spectrum_textures: HashMap<String, Arc<dyn Texture<Output=Spectrum>>>,
    named_materials: HashMap<String, Arc<dyn Material>>,

    primitives: Vec<Box<dyn Primitive>>,
    meshes: Vec<Arc<TriangleMesh>>,
    lights: Vec<Arc<dyn Light>>,
}

#[derive(Clone)]
struct GraphicsState {
    material: Option<Arc<dyn Material>>,
    area_light: Option<DiffuseAreaLightBuilder>,
    rev_orientation: bool,
}

#[derive(Debug)]
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

impl std::fmt::Display for PbrtEvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PbrtEvalError {

}

impl PbrtSceneBuilder {

    pub fn new() -> Self {
        let state = GraphicsState {
            material: None,
            area_light: None,
            rev_orientation: false,
        };
        let graphics_state = vec![state];
        let tf_state = vec![Transform::identity()];
        Self {
            graphics_state,
            tf_state,
            float_textures: Default::default(),
            spectrum_textures: Default::default(),
            named_materials: Default::default(),
            primitives: vec![],
            meshes: vec![],
            lights: vec![]
        }
    }

    pub fn create_scene(self) -> Scene {
        let bvh = BVH::build(self.primitives);
        let lights = self.lights;
        let scene = Scene::new(bvh, lights, self.meshes);
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
        let mut param_set = ParamSet { params: map };
        param_set.put_one("object_to_world".to_string(), *self.tf_state.last().unwrap());
        param_set.put_one("reverse_orientation".to_string(), vec![self.graphics_state().rev_orientation]);
        Ok(param_set)
    }

    fn current_tf_mut(&mut self) -> &mut Transform {
        self.tf_state.last_mut().expect("Transform stack empty")
    }

    fn graphics_state(&self) -> &GraphicsState {
        self.graphics_state.last().unwrap()
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
                self.light_source(name.as_ref(), params)?;
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
                let light = light.map(|l| Arc::new(l));
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
                        let light = light.map(|l| Arc::new(l));
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

            "plymesh" => {
                let mesh = make_triangle_mesh_from_ply(params)?;
                let mesh = Arc::new(mesh);
                self.meshes.push(mesh.clone());
                self.primitives.extend(mesh.iter_triangles()
                    .map(|shape| {
                        let shape = Arc::new(shape);
                        let light = graphics_state.area_light.clone()
                            .map(|builder| builder.create(shape.clone()));
                        let light = light.map(|l| Arc::new(l));
                        let material = graphics_state.material.clone();
                        let prim = GeometricPrimitive {
                            shape,
                            material,
                            light
                        };
                        Box::new(prim) as Box<dyn Primitive>
                    })
                );
            }

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
            "glass" => {
                Arc::new(make_glass(params)?)
            },
            "mirror" => {
                Arc::new(make_mirror_material(params)?)
            }
            "metal" => {
                Arc::new(make_metal_material(params)?)
            },
            "plastic" => {
                Arc::new(make_plastic_material(params)?)
            }
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
            ("spectrum", "checkerboard") | ("color", "checkerboard") => {
                let tex = make_checkerboard_spect(params)?;
                self.add_spect_tex(name.to_string(), tex);
            },
            ("spectrum", "uv") | ("color", "uv") => {
                let tex = make_uv_spect(params)?;
                self.add_spect_tex(name.to_string(), tex);
            },
            ("float", "checkerboard") => {
                let tex = make_checkerboard_float(params)?;
                self.add_float_tex(name.to_string(), tex);
            },
            ("spectrum", "imagemap") | ("color", "imagemap") => {
                let tex = make_imagemap_spect(params)?;
                self.add_spect_tex(name.to_string(), tex);
            }
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
                self.lights.push(Arc::new(light));
            },
            "distant" => {
                let light = make_distant_light(params)?;
                self.lights.push(Arc::new(light));
            },
            "infinite" => {
                let light = make_infinite_area_light(params)?;
                self.lights.push(Arc::new(light));
            }
            _ => return Err(PbrtEvalError::UnknownName(name.to_string())),
        };
        Ok(())
    }
}

pub struct PbrtHeader {
    pub tf: Transform,
    pub camera_params: ParamSet,
    camera_tf: Transform,
    sampler_params: ParamSet,
    pub film_params: ParamSet,
}

impl PbrtHeader {
    pub fn new() -> Self {
        Self {
            tf: Transform::identity(),
            camera_params: ParamSet::new(),
            camera_tf: Transform::identity(),
            sampler_params: Default::default(),
            film_params: Default::default()
        }
    }

    pub fn make_camera(&mut self) -> Result<Box<dyn Camera>, PbrtEvalError> {
        let name: String = self.camera_params.get_one("name")?;
        match name.as_ref() {
            "perspective" => {
                // According to pbrt format reference, transform statements
                // here describe the world to camera transform so we invert it.
                let cam2world = self.camera_tf.inverse();
                let fov = self.camera_params.get_one("fov").unwrap_or(90.0);
                let lens_radius = self.camera_params.get_one("lensradius").unwrap_or(0.0);
                let focal_dist = self.camera_params.get_one("focaldistance").unwrap_or(1e6);
                let shutter_open = self.camera_params.get_one("shutteropen").unwrap_or(0.0);
                let shutter_close = self.camera_params.get_one("shutterclose").unwrap_or(1.0);
                let xres = self.film_params.get_one_ref("xresolution").map(|i| *i).unwrap_or(640);
                let yres = *self.film_params.get_one_ref("yresolution").unwrap_or(&480);
                let full_resolution = Point2i::new(xres, yres);
                let frame_aspect_ratio = self.camera_params.get_one("frameaspectratio")
                    .unwrap_or(xres as f32 / yres as f32);
                let screen_window = if frame_aspect_ratio > 1.0 {
                    let pmin = Point2f::new(-frame_aspect_ratio, -1.0);
                    let pmax = Point2f::new(frame_aspect_ratio, 1.0);
                    Bounds2f::with_bounds(pmin, pmax)
                } else {
                    let pmin = Point2f::new(-1.0, -1.0 / frame_aspect_ratio);
                    let pmax = Point2f::new(1.0, 1.0 / frame_aspect_ratio);
                    Bounds2f::with_bounds(pmin, pmax)
                };

                let camera = PerspectiveCamera::new(
                    cam2world,
                    full_resolution,
                    screen_window,
                    (shutter_open, shutter_close),
                    lens_radius,
                    focal_dist,
                    fov,
                );
                Ok(Box::new(camera))
            },
            _ => Err(PbrtEvalError::UnknownName(name)),
        }
    }

    pub fn make_sampler(&mut self) -> Result<RandomSampler, PbrtEvalError> {
        let name: String = self.sampler_params.get_one("name")?;
        let spp = self.sampler_params.get_one("pixelsamples").unwrap_or(16);
        match name.as_ref() {
            "random" => {
                let sampler = RandomSampler::new_with_seed(spp as usize, 0);
                Ok(sampler)
            },
            _ => Err(PbrtEvalError::UnknownName(name))
        }
    }

    pub fn make_film(&mut self) -> Result<Film<BoxFilter>, PbrtEvalError> {
        let xres = *self.film_params.get_one_ref("xresolution").unwrap_or(&640);
        let yres = *self.film_params.get_one_ref("yresolution").unwrap_or(&480);

        let cropwindow: Vec<Float> = self.film_params.get_many("cropwindow").unwrap_or_else(|_| vec![0.0, 1.0, 0.0, 1.0]);
        let cropwindow = Bounds2f::with_bounds(
            Point2f::new(cropwindow[0], cropwindow[2]),
            Point2f::new(cropwindow[1], cropwindow[3])
        );

        let filter = BoxFilter::default();
        let film = Film::new(
            Point2i::new(xres, yres),
            cropwindow,
            filter,
            35.0
        );
        Ok(film)
    }

    pub fn exec_stmt(&mut self, stmt: parser::HeaderStmt) -> Result<(), PbrtEvalError> {
        match stmt {
            HeaderStmt::Transform(tf_stmt) => {
                self.tf = eval_transform_stmt(tf_stmt, &self.tf)?;
            },
            HeaderStmt::Camera(name, params) => {
                let mut params = Self::make_param_set(params);
                params.put_one("name".to_string(), vec![name]);
                self.camera_params = params;
                self.camera_tf = self.tf;
            },
            HeaderStmt::Sampler(name, params) => {
                let mut params = Self::make_param_set(params);
                params.put_one("name".to_string(), vec![name]);
                self.sampler_params = params;
            },
            HeaderStmt::Film(name, params) => {
                let mut params = Self::make_param_set(params);
                params.put_one("name".to_string(), vec![name]);
                self.film_params = params;
            },
            HeaderStmt::Filter(_, _) => {},
            HeaderStmt::Integrator(_, _) => {},
            HeaderStmt::Accelerator(_, _) => {},
        };
        Ok(())
    }

    fn make_param_set(params: Vec<parser::Param>) -> ParamSet {
        let map = params.into_iter()
            .map(|param| {
                let val = Self::convert_param_val(param.value);
                (param.name.to_string(), val)
            }).collect();
        ParamSet { params: map }
    }

    // TODO: convert in place!
    fn convert_param_val(val: parser::ParamVal) -> ParamVal {
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
            parser::ParamVal::Texture(s) => {
                unimplemented!()
            }
            parser::ParamVal::SpectrumRgb(v) => {
                ParamVal::Spectrum(v.into_iter().map(|s| s.into()).collect::<Vec<Spectrum>>().into())
            },
            parser::ParamVal::SpectrumXyz(_) => unimplemented!(),
            parser::ParamVal::SpectrumSampled(_) => unimplemented!(),
            parser::ParamVal::SpectrumBlackbody(_) => unimplemented!(),
        }
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
            let [angle_deg, x, y, z] = *v;
            let rot = Transform::rotate(Deg(angle_deg), Vec3f::new(x, y, z));
            *current_tf * rot
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
            Transform::from_flat(*m)
        },
        parser::TransformStmt::ConcatTransform(m) => {
            *current_tf * Transform::from_flat(*m)
        },
    };
    Ok(tf)
}



fn convert_vec<T, U: From<T>>(v: Vec<T>) -> Vec<U> {
    v.into_iter().map(Into::into).collect()
}