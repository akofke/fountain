use crate::{EFloat, Vec2f};
use crate::Float;
use crate::err_float::MACHINE_EPSILON;
use cgmath::{Matrix2, SquareMatrix};

pub const INFINITY: Float = std::f32::INFINITY;
pub const NEG_INFINITY: Float = std::f32::NEG_INFINITY;

pub fn lerp(t: Float, v1: Float, v2: Float) -> Float {
    (1.0 - t) * v1 + t * v2
}

pub fn quadratic(a: EFloat, b: EFloat, c: EFloat) -> Option<(EFloat, EFloat)> {
    let discrim: f64 = b.v as f64 * b.v as f64 - (4.0 * a.v as f64 * c.v as f64);
    if discrim < 0.0 { return None; }

    let root_discrim = discrim.sqrt();
    let root_discrim = EFloat::with_err(root_discrim as Float, MACHINE_EPSILON * root_discrim as Float);

    let q: EFloat = if b.v < 0.0 {
        -0.5 * (b - root_discrim)
    } else {
        -0.5 * (b + root_discrim)
    };

    let t0 = q / a;
    let t1 = c / q;

    if t0.v > t1.v { Some((t1, t0)) } else { Some((t0, t1)) }
}

#[allow(non_snake_case)]
pub fn solve_linear_system_2x2(A: Matrix2<Float>, b: Vec2f) -> Option<Vec2f> {
    let det = A.determinant();
    if det.abs() < 1.0e-10 {
        return None;
    }

    // A is col-major so,
    // [ A[0][0]  A[1][0]
    //   A[0][1]  A[1][1] ]
    let x0 = (A[1][1] * b[0] - A[1][0] * b[1]) / det;
    let x1 = (A[0][0] * b[1] - A[0][1] * b[0]) / det;
    if x0.is_nan() || x1.is_nan() {
        return None;
    }

    Some(Vec2f::new(x0, x1))
}

#[cfg(test)]
mod test {
    use cgmath::Matrix2;
    use crate::{Vec2f, solve_linear_system_2x2};

    #[test]
    fn test_solve_linear_system() {
        let A = Matrix2::new(3.0, 1.0, 2.0, -1.0);
        let b = Vec2f::new(5.0, 0.0);

        let res = solve_linear_system_2x2(A, b);

        assert_eq!(res, Some(Vec2f::new(1.0, 1.0)));

        let A = Matrix2::new(3.0, 1.0, 5.0, 2.0);
        let b = Vec2f::new(2.0, -1.0);

        let res = solve_linear_system_2x2(A, b);

        assert_eq!(res, Some(Vec2f::new(9.0, -5.0)));
    }
}
