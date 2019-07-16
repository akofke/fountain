use crate::{Float, Point2f, Vec2f};

pub trait Filter {
    fn evaluate(&self, p: &Point2f) -> Float;

    fn radius(&self) -> (Vec2f, Vec2f);
}

#[derive(Debug)]
pub struct BoxFilter {
    pub radius: Vec2f,
    pub inv_radius: Vec2f,
}

impl Filter for BoxFilter {
    fn evaluate(&self, p: &Point2f) -> Float {
        1.0
    }

    fn radius(&self) -> (Vec2f, Vec2f) {
        (self.radius, self.inv_radius)
    }
}

impl Default for BoxFilter {
    fn default() -> Self { 
        let radius = Vec2f::new(0.5, 0.5);
        let inv_radius = Vec2f::new(2.0, 2.0);
        Self {
            radius, inv_radius
        }
    }
}