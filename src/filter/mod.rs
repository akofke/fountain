use crate::{Float, Point2f, Vec2f};

pub trait Filter {
    fn evaluate(&self, p: &Point2f) -> Float;

    fn radius(&self) -> Vec2f;
}

pub struct BoxFilter {
    pub radius: Vec2f
}

impl Filter for BoxFilter {
    fn evaluate(&self, p: &Point2f) -> Float {
        1.0
    }

    fn radius(&self) -> Vec2f {
        self.radius
    }
}