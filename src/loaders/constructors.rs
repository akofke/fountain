use crate::loaders::{ParamSet, ParamError};
use crate::shapes::sphere::Sphere;
use crate::Transform;
use crate::material::matte::MatteMaterial;

type ParamResult<T> = Result<T, ParamError>;

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

pub fn make_matte(mut params: ParamSet) -> ParamResult<MatteMaterial> {

}