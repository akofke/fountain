use crate::Float;
use crate::spectrum::Spectrum;
use std::rc::Rc;
use crate::shapes::Shape;

pub struct DiffuseAreaLight {
    emit: Spectrum,
    shape: Rc<dyn Shape>,
    area: Float
}