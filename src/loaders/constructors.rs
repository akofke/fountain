use crate::loaders::{ParamSet, ParamError};
use crate::shapes::sphere::Sphere;
use crate::{Transform, Float, Point3f, Normal3, Vec3f, Point2f};
use crate::material::matte::MatteMaterial;
use crate::shapes::triangle::TriangleMesh;
use crate::light::diffuse::DiffuseAreaLightBuilder;
use crate::spectrum::Spectrum;
use crate::texture::checkerboard::{Checkerboard2DTexture};
use crate::texture::mapping::{TexCoordsMap2D, UVMapping};
use std::sync::Arc;
use crate::texture::{Texture, TextureRef};
use crate::light::distant::DistantLight;
use crate::light::point::PointLight;
use crate::mipmap::ImageWrap;
use crate::imageio::{ImageTexInfo, get_mipmap};
use crate::texture::image::ImageTexture;
use crate::light::infinite::InfiniteAreaLight;
use crate::material::glass::GlassMaterial;
use crate::material::metal::{MetalMaterial, RoughnessTex};
use crate::material::plastic::PlasticMaterial;
use crate::material::mirror::MirrorMaterial;
use crate::texture::uv::UVTexture;

type ParamResult<T> = Result<T, ConstructError>;

#[derive(Debug)]
pub enum ConstructError {
    ParamError(ParamError),
    ValueError(String),
}

impl From<ParamError> for ConstructError {
    fn from(e: ParamError) -> Self {
        Self::ParamError(e)
    }
}

pub fn make_sphere(mut params: ParamSet) -> ParamResult<Sphere<Transform>> {
    let radius = params.get_one("radius").unwrap_or(1.0);
    let zmin = params.get_one("zmin").unwrap_or(-radius);
    let zmax = params.get_one("zmax").unwrap_or(radius);
    let phimax = params.get_one("phimax").unwrap_or(360.0);
    let o2w = params.current_transform()?;
    let w2o = o2w.inverse();
    let rev = params.reverse_orientation()?;
    Ok(Sphere::new(
        o2w,
        w2o,
        rev,
        radius,
        zmin,
        zmax,
        phimax
    ))
}

pub fn make_triangle_mesh(mut params: ParamSet) -> ParamResult<TriangleMesh> {
    let tf = params.current_transform()?;
    let indices: Vec<i32> = params.get_one("indices")?;
    let indices = indices.into_iter().map(|i| i as u32).collect();
    let vertices = params.get_one("P")?;
    let normals = params.get_one("N").ok();
    let tangents = params.get_one("S").ok();
    // TODO: handle float array
    let tex_coords = params.get_one("uv")
        .or_else(|_| params.get_one("st"))
        .ok()
        .or_else(|| {
            params
                .get_one::<Vec<Float>>("uv")
                .or_else(|_| params.get_one("st"))
                .ok()
                .filter(|v| v.len() % 2 == 0)
                .map(|uvs| {
                    uvs.chunks_exact(2)
                        .map(|uv| Point2f::new(uv[0], uv[1]))
                        .collect()
                })
        });
    dbg!(&tex_coords);
    let reverse_orientation = params.reverse_orientation()?;

    let mesh = TriangleMesh::new(
        tf,
        indices,
        vertices,
        normals,
        tangents,
        tex_coords,
        reverse_orientation
    );
    Ok(mesh)
}

pub fn make_triangle_mesh_from_ply(mut params: ParamSet) -> ParamResult<TriangleMesh> {
    use ply_rs as ply;
    use ply::ply::Property;

    let tf = params.current_transform()?;
    let rev = params.reverse_orientation()?;
    let filename: String = params.get_one("filename")?;
    let mut f = std::fs::File::open(filename).unwrap();
    let parser = ply::parser::Parser::<ply::ply::DefaultElement>::new();
    let plyfile = parser.read_ply(&mut f).unwrap();

    let vertices: Vec<Point3f> = plyfile.payload["vertex"].iter()
        .map(|el| {
            match (&el["x"], &el["y"], &el["z"]) {
                (Property::Float(x), Property::Float(y), Property::Float(z)) => {
                    Point3f::new(*x, *y, *z)
                },
                _ => panic!(),
            }
        })
        .collect();

    let normals: Option<Vec<Normal3>> = plyfile.payload["vertex"].iter()
        .map(|el| {
            match (&el.get("nx"), &el.get("ny"), &el.get("nz")) {
                (Some(Property::Float(x)), Some(Property::Float(y)), Some(Property::Float(z))) => {
                    Some(Normal3(Vec3f::new(*x, *y, *z)))
                },
                _ => None,
            }
        })
        .collect();

    let tex_coords: Option<Vec<Point2f>> = plyfile.payload["vertex"].iter()
        .map(|el| {
            match (&el.get("v"), &el.get("v")) {
                (Some(Property::Float(u)), Some(Property::Float(v))) => {
                    Some(Point2f::new(*u, *v))
                },
                _ => None,
            }
        })
        .collect();

    let indices: Vec<u32> = plyfile.payload["face"].iter()
        .flat_map(|el| {
            match &el["vertex_indices"] {
                Property::ListInt(v) if v.len() == 3 => {
                    v
                },
                Property::ListInt(v) => panic!("Face with unsupported vertex count {} found", v.len()),
                p @ _ => panic!("{:?}", p)
            }
        })
        .map(|i| *i as u32)
        .collect();

    let mesh = TriangleMesh::new(
        tf,
        indices,
        vertices,
        normals,
        None,
        tex_coords,
        rev
    );
    Ok(mesh)
}

pub fn make_matte(mut params: ParamSet) -> ParamResult<MatteMaterial> {
    let diffuse = params.get_texture_or_const("Kd")?;
    let sigma = params.get_texture_or_default("sigma", 0.0)?;
    Ok(MatteMaterial::new(diffuse, sigma))
}

pub fn make_glass(mut params: ParamSet) -> ParamResult<GlassMaterial> {
    let kr = params.get_texture_or_default("Kr", Spectrum::uniform(1.0))?;
    let kt = params.get_texture_or_default("Kt", Spectrum::uniform(1.0))?;
    let urough = params.get_texture_or_default("uroughness", 0.0)?;
    let vrough = params.get_texture_or_default("vroughness", 0.0)?;
    let eta = params.get_texture_or_default("eta", 1.5)?;
    let remap = params.get_one("remaproughness").unwrap_or(true);
    Ok(GlassMaterial::new(kr, kt, urough, vrough,  eta, remap))
}

pub fn make_mirror_material(mut params: ParamSet) -> ParamResult<MirrorMaterial> {
    let kr = params.get_texture_or_default("Kr", Spectrum::uniform(0.9))?;
    Ok(MirrorMaterial::new(kr))
}

pub fn make_metal_material(mut params: ParamSet) -> ParamResult<MetalMaterial> {
    // TODO: defaults?
    let eta = params.get_texture_or_const("eta")?;
    let k = params.get_texture_or_const("k")?;
    let roughness = params.get_texture_or_default("roughness", 0.01)?;
    let u_rough = params.get_texture_or_const("uroughness");
    let v_rough = params.get_texture_or_const("vroughness");
    let rough_tex = match (u_rough, v_rough) {
        (Ok(u_rough), Ok(v_rough)) => {
            RoughnessTex::Anisotropic { u_rough, v_rough }
        },
        _ => RoughnessTex::Isotropic(roughness)
    };

    let remap = params.get_one("remaproughness").unwrap_or(true);

    Ok(MetalMaterial::new(eta, k, rough_tex, remap))
}

pub fn make_plastic_material(mut params: ParamSet) -> ParamResult<PlasticMaterial> {
    let kd = params.get_texture_or_default("Kd", Spectrum::uniform(0.25))?;
    let ks = params.get_texture_or_default("ks", Spectrum::uniform(0.25))?;
    let roughness = params.get_texture_or_default("roughness", 0.1)?;
    let remap = params.get_one("remaproughness").unwrap_or(true);
    Ok(PlasticMaterial::new(kd, ks, roughness, remap))
}

pub fn make_diffuse_area_light(mut params: ParamSet) -> ParamResult<DiffuseAreaLightBuilder> {
    let emit = params.get_one("L").unwrap_or(Spectrum::uniform(1.0));
    let _two_sided = params.get_one("twosided").unwrap_or(false);
    let samples = params.get_one("samples").unwrap_or(1) as usize;
    Ok(DiffuseAreaLightBuilder { emit, n_samples: samples })
}

fn make_tex_coords_map_2d(params: &mut ParamSet) -> Result<Arc<dyn TexCoordsMap2D>, ConstructError> {
    let map_type = params.get_one("mapping").unwrap_or_else(|_| "uv".to_string());
    match map_type.as_ref() {
        "uv" => {
            let uscale = params.get_one("uscale").unwrap_or(1.0);
            let vscale = params.get_one("vscale").unwrap_or(1.0);
            let udelta = params.get_one("udelta").unwrap_or(0.0);
            let vdelta = params.get_one("vdelta").unwrap_or(0.0);
            let map = UVMapping::new(uscale, vscale, udelta, vdelta);
            Ok(Arc::new(map))
        }
        _ => Err(ConstructError::ValueError(format!("Unknown mapping type {}", map_type)))
    }

}

pub fn make_checkerboard_float(mut params: ParamSet) -> ParamResult<Arc<dyn Texture<Output=Float>>> {
    let mapping = make_tex_coords_map_2d(&mut params)?;
    let tex1 = params.get_texture_or_const::<Float>("tex1")?;
    let tex2 = params.get_texture_or_const::<Float>("tex2")?;

    let tex = Arc::new(Checkerboard2DTexture::new(
        tex1,
        tex2,
        mapping
    ));
    Ok(tex)
}

pub fn make_checkerboard_spect(mut params: ParamSet) -> ParamResult<Arc<dyn Texture<Output=Spectrum>>> {
    let mapping = make_tex_coords_map_2d(&mut params)?;
    let tex1 = params.get_texture_or_const::<Spectrum>("tex1")?;
    let tex2 = params.get_texture_or_const::<Spectrum>("tex2")?;

    let tex = Arc::new(Checkerboard2DTexture::new(
        tex1,
        tex2,
        mapping
    ));
    Ok(tex)
}

pub fn make_uv_spect(mut params: ParamSet) -> ParamResult<TextureRef<Spectrum>> {
    let mapping = make_tex_coords_map_2d(&mut params)?;
    let tex = Arc::new(UVTexture::new(mapping));
    Ok(tex)
}

pub fn make_imagemap_spect(mut params: ParamSet) -> ParamResult<Arc<dyn Texture<Output=Spectrum>>> {
    let filename = params.get_one("filename")?;
    let wrap_mode = params.get_one("wrap").or_else(|_| Ok("repeat".to_string())).and_then(|s| {
        match s.as_ref() {
            "repeat" => Ok(ImageWrap::Repeat),
            "black" => Ok(ImageWrap::Black),
            "clamp" => Ok(ImageWrap::Clamp),
            _ => Err(ConstructError::ValueError(format!("Unknown repeat type {}", s)))
        }
    })?;
    let mapping = make_tex_coords_map_2d(&mut params)?;
    let scale = params.get_one("scale").unwrap_or(1.0);
    let gamma = true; // FIXME: depends on format
    let info = ImageTexInfo::new(
        filename,
        wrap_mode,
        scale,
        gamma
    );
    let mipmap = get_mipmap(info).unwrap(); // FIXME: propagate error
    let tex = Arc::new(ImageTexture::new(mapping, mipmap));
    Ok(tex)
}

pub fn make_distant_light(mut params: ParamSet) -> ParamResult<DistantLight> {
    let radiance = params.get_one("L").unwrap_or(Spectrum::uniform(1.0));
    let scale = params.get_one("scale").unwrap_or(Spectrum::uniform(1.0));
    let radiance = radiance * scale;
    let from = params.get_one("from").unwrap_or(Point3f::new(0.0, 0.0, 0.0));
    let to = params.get_one("to").unwrap_or(Point3f::new(0.0, 0.0, 1.0));
    Ok(DistantLight::from_to(from, to, radiance))
}

pub fn make_point_light(mut params: ParamSet) -> ParamResult<PointLight> {
    let intensity = params.get_one("I").unwrap_or(Spectrum::uniform(1.0));
    let scale = params.get_one("scale").unwrap_or(Spectrum::uniform(1.0));
    let intensity = intensity * scale;
    let from = params.get_one("from").unwrap_or(Point3f::new(0.0, 0.0, 0.0));
    let light_to_world = Transform::translate(from - Point3f::new(0.0, 0.0, 0.0));
    Ok(PointLight::new(light_to_world, intensity))
}

pub fn make_infinite_area_light(mut params: ParamSet) -> ParamResult<InfiniteAreaLight> {
    let radiance = params.get_one("L").unwrap_or(Spectrum::uniform(1.0));
    let filename = params.get_one::<String>("mapname");
    let l2w = params.current_transform()?;
    let light = filename.map_or_else(
        |_| InfiniteAreaLight::new_uniform(radiance, l2w),
        |filename| {
            let info = ImageTexInfo::new(
                filename,
                ImageWrap::Repeat,
                1.0,
                false, // TODO: pbrt never gamma corrects here
            );
            let mipmap = get_mipmap(info).unwrap();
            InfiniteAreaLight::new_envmap(mipmap, l2w)
        }
    );
    Ok(light)
}
