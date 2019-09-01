use crate::{Point3f, Transform, Bounds3f, Ray, Float, SurfaceInteraction, Normal3, Vec3f, Point2f, ComponentWiseExt, max_dimension, permute_vec, permute_point};
use std::sync::Arc;
use std::convert::TryInto;
use crate::shapes::Shape;
use cgmath::EuclideanSpace;
use crate::interaction::DiffGeom;

pub struct TriangleMesh {
    pub n_triangles: u32,

    vertex_indices: Vec<u32>,

    vertices: Vec<Point3f>,

    normals: Option<Vec<Normal3>>,

    tangents: Option<Vec<Vec3f>>,

    tex_coords: Option<Vec<Point2f>>,
}

impl TriangleMesh {
    pub fn new(
        object_to_world: &Transform,
        vertex_indices: Vec<u32>,
        mut vertices: Vec<Point3f>,
        mut normals: Option<Vec<Normal3>>,
        mut tangents: Option<Vec<Vec3f>>,
        mut tex_coords: Option<Vec<Point2f>>,
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
            tex_coords
        }
    }
}

pub struct Triangle<'m> {
    mesh: &'m TriangleMesh,
    tri_id: u32,
    vertex_indices: &'m [u32; 3],
}

impl<'m> Triangle<'m> {
    pub fn new(mesh: &'m TriangleMesh, tri_id: u32) -> Self {
        let idx = tri_id as usize;
        let vertex_indices: &[u32; 3] = mesh.vertex_indices[idx .. idx + 3].try_into().unwrap();

        Self {
            mesh,
            tri_id,
            vertex_indices
        }
    }

    fn get_uvs(&self) -> [Point2f; 3] {
        self.mesh.tex_coords.as_ref().map_or_else(
            || [(0.0, 0.0).into(), (1.0, 0.0).into(), (1.0, 1.0).into()],
            |uvs| {
                let v = self.vertex_indices;
                [
                    uvs[v[0] as usize],
                    uvs[v[1] as usize],
                    uvs[v[2] as usize]
                ]
            }
        )
    }
}

impl<'m> Shape for Triangle<'m> {
    fn object_bound(&self) -> Bounds3f {
        unimplemented!()
    }

    fn world_bound(&self) -> Bounds3f {
        unimplemented!()
    }

    fn object_to_world(&self) -> &Transform {
        unimplemented!()
    }

    fn world_to_object(&self) -> &Transform {
        unimplemented!()
    }

    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)> {
        let v = self.vertex_indices;
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
        let e0 = p1t.x * p2t.y - p1t.y * p2t.x; // p1 to p2
        let e1 = p2t.x * p0t.y - p2t.y * p0t.x; // p2 to p0
        let e2 = p0t.x * p1t.y - p0t.y * p1t.x; // p0 to p1

        // TODO: fall back on double precision

        // if one of the edge function signs differs, then the point (0, 0) is not on the same side
        // of all three edges so therefore is outside the triangle.
        let sign_differs =
            (e0.is_sign_positive(), e1.is_sign_positive()) == (e1.is_sign_positive(), e2.is_sign_positive());

        if sign_differs { return None; }

        let det = e0 + e1 + e2;
        if det == 0.0 { return None; }

        // Compute scaled hit distance to triangle and test against ray t range
        p0t.z *= shear_z;
        p1t.z *= shear_z;
        p2t.z *= shear_z;
        let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
        if det < 0.0 && (t_scaled >= 0.0 || t_scaled < ray.t_max * det) { // TODO: can probably optimize (is_sign_pos)
            return None;
        } else if det > 0.0 && (t_scaled <= 0.0 || t_scaled > ray.t_max * det) {
            return None;
        }

        // now we know there is a valid intersection.
        // compute barycentric coordinates and actual t value.
        let inv_det = 1.0 / det;
        let b0 = e0 * inv_det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;
        let t = t_scaled * inv_det;

        // compute triangle partial derivatives.
        let uv = self.get_uvs();
        let duv02 = uv[0] - uv[2];
        let duv12 = uv[1] - uv[2];
        let dp02 = p0 - p2;
        let dp12 = p1 - p2;

        let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];

        let (dpdu, dpdv) = if determinant == 0.0 {
            unimplemented!(); // TODO
        } else {
            let inv_det = 1.0 / determinant;
            let dpdu = (duv12[1] * dp02 - duv02[1] * dp12) * inv_det;
            let dpdv = (-duv12[0] * dp02 + duv02[0] * dp12) * inv_det;
            (dpdu, dpdv)
        };

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

        let p_err = Vec3f::new(0.0, 0.0, 0.0);
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
        unimplemented!()
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        unimplemented!()
    }
}