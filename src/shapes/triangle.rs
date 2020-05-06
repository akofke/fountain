use crate::{Point3f, Transform, Bounds3f, Ray, Float, SurfaceInteraction, Normal3, Vec3f, Point2f, ComponentWiseExt, max_dimension, permute_vec, permute_point, coordinate_system, faceforward};
use std::sync::Arc;
use crate::shapes::Shape;
use cgmath::{EuclideanSpace, InnerSpace};
use crate::interaction::{DiffGeom, SurfaceHit};
use crate::err_float::gamma;
use crate::sampling::uniform_sample_triangle;

#[derive(Debug)]
pub struct TriangleMesh {
    pub n_triangles: u32,

    vertex_indices: Vec<u32>,

    vertices: Vec<Point3f>,

    normals: Option<Vec<Normal3>>,

    tangents: Option<Vec<Vec3f>>,

    tex_coords: Option<Vec<Point2f>>,

    reverse_orientation: bool,

    object_to_world: Transform,
}

impl TriangleMesh {
    pub fn new(
        object_to_world: Transform,
        vertex_indices: Vec<u32>,
        mut vertices: Vec<Point3f>,
        mut normals: Option<Vec<Normal3>>,
        mut tangents: Option<Vec<Vec3f>>,
        tex_coords: Option<Vec<Point2f>>,
        reverse_orientation: bool,
    ) -> Self {
        assert_eq!(vertex_indices.len() % 3, 0);
        let n_triangles = vertex_indices.len() as u32 / 3;
        let n_vertices = vertices.len();

        for v in &mut vertices {
            *v = object_to_world.transform(*v);
        }

        if let Some(ref mut normals) = normals {
            assert_eq!(normals.len(), n_vertices);
            for n in normals {
                *n = object_to_world.transform(*n);
            }
        }

        if let Some(ref mut tangents) = tangents {
            assert_eq!(tangents.len(), n_vertices);
            for t in tangents {
                *t = object_to_world.transform(*t);
            }
        }

        if let Some(ref tex_coords) = tex_coords {
            assert_eq!(tex_coords.len(), n_vertices);
        }

        Self {
            n_triangles,
            vertex_indices,
            vertices,
            normals,
            tangents,
            tex_coords,
            reverse_orientation,
            object_to_world
        }
    }

    pub fn iter_triangles(self: Arc<Self>) -> impl Iterator<Item=Triangle> {
        (0..self.n_triangles).map(move |tri_id| {
            Triangle::new(Arc::clone(&self), tri_id)
        })
    }
}

pub struct Triangle {
    mesh: Arc<TriangleMesh>,
    tri_id: u32,
}

impl Triangle {
    pub fn new(mesh: Arc<TriangleMesh>, tri_id: u32) -> Self {
        Self {
            mesh,
            tri_id,
        }
    }

    fn vertex_indices(&self) -> [u32; 3] {
        let idx = self.tri_id as usize;
        [
            self.mesh.vertex_indices[3 * idx],
            self.mesh.vertex_indices[3 * idx + 1],
            self.mesh.vertex_indices[3 * idx + 2],
        ]
    }

    fn get_vertices(&self) -> [Point3f; 3] {
        let v = self.vertex_indices();
        let p0 = self.mesh.vertices[v[0] as usize];
        let p1 = self.mesh.vertices[v[1] as usize];
        let p2 = self.mesh.vertices[v[2] as usize];
        [p0, p1, p2]
    }

    fn get_vertices_as_vectors(&self) -> [Vec3f; 3] {
        let v = self.vertex_indices();
        let p0 = self.mesh.vertices[v[0] as usize];
        let p1 = self.mesh.vertices[v[1] as usize];
        let p2 = self.mesh.vertices[v[2] as usize];
        [p0.to_vec(), p1.to_vec(), p2.to_vec()]
    }

    fn get_normals(&self) -> Option<[Normal3; 3]> {
        self.mesh.normals.as_ref().map(|normals| {
            let v = self.vertex_indices();
            let n0 = normals[v[0] as usize];
            let n1 = normals[v[1] as usize];
            let n2 = normals[v[2] as usize];
            [n0, n1, n2]
        })
    }

    fn get_uvs(&self) -> [Point2f; 3] {
        self.mesh.tex_coords.as_ref().map_or_else(
            || [(0.0, 0.0).into(), (1.0, 0.0).into(), (1.0, 1.0).into()],
            |uvs| {
                let v = self.vertex_indices();
                [
                    uvs[v[0] as usize],
                    uvs[v[1] as usize],
                    uvs[v[2] as usize]
                ]
            }
        )
    }
}

impl Shape for Triangle {
    fn object_bound(&self) -> Bounds3f {
        unimplemented!()
    }

    fn world_bound(&self) -> Bounds3f {
        let v = self.vertex_indices();
        let p0 = self.mesh.vertices[v[0] as usize];
        let p1 = self.mesh.vertices[v[1] as usize];
        let p2 = self.mesh.vertices[v[2] as usize];
        Bounds3f::empty().join_point(p0).join_point(p1).join_point(p2)
    }

    fn object_to_world(&self) -> &Transform {
        &self.mesh.object_to_world
    }

    fn world_to_object(&self) -> &Transform {
        unimplemented!()
    }

    fn reverse_orientation(&self) -> bool {
        self.mesh.reverse_orientation
    }

    fn area(&self) -> Float {
        let [p0, p1, p2] = self.get_vertices();
        0.5 * (p1 - p0).cross(p2 - p0).magnitude()
    }

    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)> {
        let v = self.vertex_indices();
        let p0 = self.mesh.vertices[v[0] as usize];
        let p1 = self.mesh.vertices[v[1] as usize];
        let p2 = self.mesh.vertices[v[2] as usize];

        // First compute an affine transformation that transforms the ray such that its origin is at
        // (0, 0, 0) and points along the +z axis.

        // translate vertices based on ray origin.
        let mut p0t = p0 - ray.origin.to_vec();
        let mut p1t = p1 - ray.origin.to_vec();
        let mut p2t = p2 - ray.origin.to_vec();

        // permute components of triangle vertices and ray dir
        let kz = max_dimension(ray.dir.abs());
        let kx = (kz + 1) % 3;
        let ky = (kx + 1) % 3;
        let dir = permute_vec(ray.dir, kx, ky, kz);
        p0t = permute_point(p0t, kx, ky, kz);
        p1t = permute_point(p1t, kx, ky, kz);
        p2t = permute_point(p2t, kx, ky, kz);

        // Apply a shear transformation to align the ray with the +z axis.
        // Only shear the x and y dimensions of the vertices at first, wait to apply the z shear
        // if the ray actually intersects the triangle.
        // TODO: cache shear coefficients in ray
        let shear_x = -dir.x / dir.z;
        let shear_y = -dir.y / dir.z;
        let shear_z = 1.0 / dir.z;
        p0t.x += shear_x * p0t.z;
        p0t.y += shear_y * p0t.z;
        p1t.x += shear_x * p1t.z;
        p1t.y += shear_y * p1t.z;
        p2t.x += shear_x * p2t.z;
        p2t.y += shear_y * p2t.z;

        // compute edge function coefficients
        let mut e0 = p1t.x * p2t.y - p1t.y * p2t.x; // p1 to p2
        let mut e1 = p2t.x * p0t.y - p2t.y * p0t.x; // p2 to p0
        let mut e2 = p0t.x * p1t.y - p0t.y * p1t.x; // p0 to p1

        // TODO: fall back on double precision
        if e0 == 0.0 || e1 == 0.0 || e2 == 0.0 {
            e0 = (p1t.x as f64 * p2t.y as f64 - p1t.y as f64 * p2t.x as f64) as Float; // p1 to p2
            e1 = (p2t.x as f64 * p0t.y as f64 - p2t.y as f64 * p0t.x as f64) as Float; // p2 to p0
            e2 = (p0t.x as f64 * p1t.y as f64 - p0t.y as f64 * p1t.x as f64) as Float; // p0 to p1
        }

        // if one of the edge function signs differs, then the point (0, 0) is not on the same side
        // of all three edges so therefore is outside the triangle.
        if sign_differs(e0, e1, e2) { return None; }

        let det = e0 + e1 + e2;
        if det == 0.0 { return None; }

        // Compute scaled hit distance to triangle and test against ray t range
        p0t.z *= shear_z;
        p1t.z *= shear_z;
        p2t.z *= shear_z;
        let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
        if det < 0.0 && (t_scaled >= 0.0 || t_scaled < ray.t_max * det)
            || det > 0.0 && (t_scaled <= 0.0 || t_scaled > ray.t_max * det)
        {
            // TODO: can probably optimize (is_sign_pos)
            return None;
        }

        // now we know there is a valid intersection.
        // compute barycentric coordinates and actual t value.
        let inv_det = 1.0 / det;
        let b0 = e0 * inv_det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;
        let t = t_scaled * inv_det;

        // Ensure that the computed triangle t is conservatively greater than 0.
        // Compute delta_z term for triangle t error bounds.
        let max_zt = p0t.z.abs().max(p1t.z.abs()).max(p2t.z.abs());
        let delta_z = gamma(3) * max_zt;

        // Compute delta_x and delta_y terms for triangle t error bounds.
        let max_xt = p0t.x.abs().max(p1t.x.abs()).max(p2t.x.abs());
        let max_yt = p0t.y.abs().max(p1t.y.abs()).max(p2t.y.abs());
        let delta_x = gamma(5) * (max_xt + max_zt);
        let delta_y = gamma(5) * (max_yt + max_zt);

        let delta_e = 2.0 * (gamma(2) * max_xt * max_yt + delta_y * max_xt + delta_x * max_yt);

        let max_e = e0.abs().max(e1.abs()).max(e2.abs());
        let delta_t = 3.0 * (gamma(3) * max_e * max_zt + delta_e * max_zt + delta_z * max_e) *
            inv_det.abs();
        if t <= delta_t { return None; }

        // compute triangle partial derivatives.
        let uv = self.get_uvs();
        let duv02 = uv[0] - uv[2];
        let duv12 = uv[1] - uv[2];
        let dp02 = p0 - p2;
        let dp12 = p1 - p2;

        let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        let degenerate_uv = determinant.abs() < 1.0e-8;

        let (dpdu, dpdv) = if degenerate_uv {
            // TODO: zero-length normal
            let ng = (p2 - p0).cross(p1 - p0);
            if ng.magnitude2() == 0.0 {
                // triangle is actually degenerate
                return None;
            } else {
                coordinate_system(ng.normalize())
            }
        } else {
            let inv_det = 1.0 / determinant;
            let dpdu = (duv12[1] * dp02 - duv02[1] * dp12) * inv_det;
            let dpdv = (-duv12[0] * dp02 + duv02[0] * dp12) * inv_det;
            (dpdu, dpdv)
        };

        // Compute error bounds for triangle intersection point
        let x_abs_sum = (b0 * p0.x).abs() + (b1 * p1.x).abs() + (b2 * p2.x).abs();
        let y_abs_sum = (b0 * p0.y).abs() + (b1 * p1.y).abs() + (b2 * p2.y).abs();
        let z_abs_sum = (b0 * p0.z).abs() + (b1 * p1.z).abs() + (b2 * p2.z).abs();
        let p_err = gamma(7) * Vec3f::new(x_abs_sum, y_abs_sum, z_abs_sum);

        // interpolate uv coordinates and hit point using barycentric coordinates
        let p_hit = Point3f::from_vec(b0 * p0.to_vec() + b1 * p1.to_vec() + b2 * p2.to_vec());
        let uv_hit = Point2f::from_vec(b0 * uv[0].to_vec() + b1 * uv[1].to_vec() + b2 * uv[2].to_vec());

        // TODO: alpha mask

        let diff_geom = DiffGeom {
            dpdu,
            dpdv,
            dndu: Normal3::new(0.0, 0.0, 0.0),
            dndv: Normal3::new(0.0, 0.0, 0.0),
        };

        let geom_normal = Normal3(dp02.cross(dp12).normalize());

        let mut isect = SurfaceInteraction::new(
            p_hit,
            p_err,
            ray.time,
            uv_hit,
            -ray.dir,
            geom_normal,
            diff_geom
        );

        if self.flip_normals() {
            isect.hit.n *= -1.0;
            isect.shading_n *= -1.0;
        }

        if self.mesh.normals.is_some() || self.mesh.tangents.is_some() {
            // compute shading normal
            let ns = if let Some(normals) = &self.mesh.normals {
                Normal3((b0 * normals[v[0] as usize] + b1 * normals[v[1] as usize] + b2 * normals[v[2] as usize]).normalize())
            } else {
                isect.hit.n
            };

            // compute shading tangent
            let ss = if let Some(tangents) = &self.mesh.tangents {
                (b0 * tangents[v[0] as usize] + b1 * tangents[v[1] as usize] + b2 * tangents[v[2] as usize]).normalize()
            } else {
                isect.geom.dpdu.normalize()
            };

            // compute shading bitangent ts and adjust shading tangent ss
            let ts = ns.cross(ss);
            let (ts, ss) = if ts.magnitude2() > 0.0 {
                let ts = ts.normalize();
                let ss = ts.cross(ns.0);
                (ts, ss)
            } else {
                coordinate_system(ns.0)
            };

            let (dndu, dndv) = if let Some(normals) = &self.mesh.normals {
                let dn1 = normals[v[0] as usize] - normals[v[2] as usize];
                let dn2 = normals[v[1] as usize] - normals[v[2] as usize];

                if degenerate_uv {
                    let dn = (normals[v[2] as usize] - normals[v[0] as usize]).0
                        .cross((normals[v[1] as usize] - normals[v[0] as usize]).0);
                    if dn.magnitude2() == 0.0 {
                        (Normal3::new(0.0, 0.0, 0.0), Normal3::new(0.0, 0.0, 0.0))
                    } else {
                        let (dndu, dndv) = coordinate_system(dn);
                        (Normal3(dndu), Normal3(dndv))
                    }
                } else {
                    let dndu = (duv12[1] * dn1 - duv02[1] * dn2) * inv_det;
                    let dndv = (-duv12[0] * dn1 + duv02[0] * dn2) * inv_det;
                    (dndu, dndv)
                }
            } else {
                (Normal3::new(0.0, 0.0, 0.0), Normal3::new(0.0, 0.0, 0.0))
            };

            let shading_geom = DiffGeom {
                dpdu: ss,
                dpdv: ts,
                dndu,
                dndv,
            };
            isect.shading_geom = shading_geom;

            isect.shading_n = ns;

            // TODO: clean up orientation
            isect.hit.n = Normal3(faceforward(isect.hit.n.0, isect.shading_n.0));
        }
        Some((t, isect))
    }

    fn sample(&self, u: Point2f) -> SurfaceHit {
        let b = uniform_sample_triangle(u);
        let [p0, p1, p2] = self.get_vertices_as_vectors();
        let sample_p = b[0] * p0 + b[1] * p1 + (1.0 - b[0] - b[1]) * p2;

        let n = Normal3((p1 - p0).cross(p2 - p0).normalize());

        let sample_n = if let Some([n0, n1, n2]) = self.get_normals() {
            let ns = Normal3((b[0] * n0 + b[1] * n1 + (1.0 - b[0] - b[1]) * n2).normalize());
            faceforward(n.0, ns.0).into()
        } else if self.flip_normals() {
            n * -1.0
        } else {
            n
        };

        let p_abs_sum = (b[0] * p0).abs() + (b[1] * p1).abs() + ((1.0 - b[0] - b[1]) * p2).abs();
        let p_err = gamma(6) * p_abs_sum;
        
        SurfaceHit {
            p: Point3f::new(0.0, 0.0, 0.0) + sample_p,
            p_err,
            time: 0.0,
            n: sample_n
        }
    }

//    fn intersect_test(&self, ray: &Ray) -> bool {
//        false
//    }
}

#[inline]
fn sign_differs(v1: Float, v2: Float, v3: Float) -> bool {
    // This is the original implementation from the book; however below generates better assembly.
    // They differ in results for pos/neg 0.0, but in the triangle intersection this is already handled
    // (v1 < 0.0 || v2 < 0.0 || v3 < 0.0) && (v1 > 0.0 || v2 > 0.0 || v3 > 0.0)

    v1.is_sign_positive() != v2.is_sign_positive() || v2.is_sign_positive() != v3.is_sign_positive()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_differs() {
        assert_eq!(sign_differs(1.0, 2.0, -1.0), true);
        assert_eq!(sign_differs(1.0, 2.0, 1.0), false);
        assert_eq!(sign_differs(-1.0, -2.0, 1.0), true);
        assert_eq!(sign_differs(-1.0, -2.0, -1.0), false);
        assert_eq!(sign_differs(-1.0, 2.0, -1.0), true);
        assert_eq!(sign_differs(-1.0, 2.0, 1.0), true);
        assert_eq!(sign_differs(0.0, 0.0, 0.0), false);
        assert_eq!(sign_differs(0.0, 0.0, -0.0), true);
    }

    #[test]
    fn test_tri_isect() {

    }
}