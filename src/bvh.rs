use arrayvec::ArrayVec;
use bumpalo::Bump;

use partition::partition;

use crate::{Ray, SurfaceInteraction};
use crate::geometry::bounds::Bounds3f;
use crate::Point3f;
use crate::primitive::Primitive;

#[derive(Copy, Clone)]
pub enum SplitMethod {
    Middle,
    EqualCounts,
    SAH
}

pub struct BVH<P: AsRef<dyn Primitive> = Box<dyn Primitive>> {
    pub prims: Vec<P>,
    pub bounds: Bounds3f,
    nodes: Vec<LinearBVHNode>
}

impl<P: AsRef<dyn Primitive>> BVH<P> {
    pub fn build(mut prims: Vec<P>) -> Self {
        // TODO: figure out prims type. Rc or Box?

        if prims.is_empty() {
            return BVH { prims, bounds: Bounds3f::empty(), nodes: Vec::new() }
        }

        let mut prim_info: Vec<BVHPrimInfo> = prims.iter().enumerate().map(|(i, p)| {
            BVHPrimInfo::new(i, p.as_ref().world_bound())
        }).collect();

        let arena = Bump::new();
        let mut prim_ordering: Vec<isize> = Vec::with_capacity(prims.len());

        let root = Self::recursive_build(
            &arena,
            &mut prim_info,
            &mut prim_ordering,
            SplitMethod::Middle
        );

        let world_bound = root.bounds();

        apply_permutation(&mut prims, &mut prim_ordering);

        let mut flat_nodes = Vec::<LinearBVHNode>::with_capacity(prims.len());

        let tree_len = Self::flatten_tree(&mut flat_nodes, root);
        assert_eq!(flat_nodes.len(), tree_len);
        BVH {
            prims,
            bounds: world_bound,
            nodes: flat_nodes
        }
    }

    fn recursive_build<'a>(
        arena: &'a Bump,
        prim_info: &mut [BVHPrimInfo],
//        range: Range<usize>,
        prim_ordering: &mut Vec<isize>,
        split_method: SplitMethod
    ) -> &'a BVHBuildNode<'a> {

        // Find the union of the bounding boxes of all primitives in this node,
        // and the bounding box of all centroids
        let (node_bounds, centroid_bounds) = prim_info.iter()
            .fold((Bounds3f::empty(), Bounds3f::empty()), |(node_bb, centr_bb), prim| {
                (node_bb.join(&prim.bounds), centr_bb.join_point(prim.centroid))
            });

        let n_prims = prim_info.len();

        // If there is only one primitive or all the centroids lie on the same point
        // (and therefore can't be partitioned), create a leaf node.
        if n_prims == 1 || centroid_bounds.is_point() {
            let first_prim_idx = prim_ordering.len();
            for prim in prim_info {
                prim_ordering.push(prim.prim_id as isize)
            }
            let node = arena.alloc(
                BVHBuildNode::new_leaf(first_prim_idx as u32, n_prims as u16, node_bounds));
            return node;
        }

        let ax = centroid_bounds.maximum_extent() as usize;

        let mid = prim_info.len() / 2;

        let (part1, part2) = match split_method {
            SplitMethod::Middle => {
                let midpoint = (centroid_bounds.min[ax] + centroid_bounds.max[ax]) / 2.0;
                let (part1, part2) = partition(prim_info, |prim| {
                    prim.centroid[ax] < midpoint
                });
                if part1.len() == 0 || part2.len() == 0 {
                    Self::partition_equal_counts(prim_info, ax)
                } else {
                    (part1, part2)
                }
            },

            SplitMethod::EqualCounts => {
                Self::partition_equal_counts(prim_info, ax)
            }
            _ => unimplemented!()
        };

        let child1 = Self::recursive_build(arena, part1, prim_ordering, split_method);
        let child2 = Self::recursive_build(arena, part2, prim_ordering, split_method);

        arena.alloc(BVHBuildNode::new_interior([child1, child2], ax as u8))
    }

    fn partition_equal_counts(prim_info: &mut [BVHPrimInfo], ax: usize)
        -> (&mut [BVHPrimInfo], &mut [BVHPrimInfo])
    {
        let mid = prim_info.len() / 2;
        prim_info.partition_at_index_by(mid, |a, b| {
            a.centroid[ax].partial_cmp(&b.centroid[ax]).unwrap()
        });
        prim_info.split_at_mut(mid)
    }

    // Returns subtree length
    fn flatten_tree(flat_nodes: &mut Vec<LinearBVHNode>, node: &BVHBuildNode) -> usize {
        let subtree_len = match *node {
            BVHBuildNode::Leaf {bounds, first_prim_idx, n_prims} => {
                let leaf = LinearBVHNode::new_leaf(bounds, first_prim_idx, n_prims);
                flat_nodes.push(leaf);
                1
            },

            BVHBuildNode::Interior {bounds, children, split_axis} => {
                let interior = LinearBVHNode::new_interior(bounds, 0, split_axis);
                flat_nodes.push(interior);
                let my_idx = flat_nodes.len() - 1;
                let first_subtree_len = Self::flatten_tree(flat_nodes, children[0]);
                let second_idx = my_idx + first_subtree_len + 1;
                if let LinearNodeKind::Interior {ref mut second_child_idx, ..} = flat_nodes[my_idx].kind {
                    *second_child_idx = second_idx as u32;
                } else { unreachable!() } // unchecked?

                let second_subtree_len = Self::flatten_tree(flat_nodes, children[1]);
                // The length of this subtree is the length of this interior node's child subtrees
                // plus one for this node
                first_subtree_len + second_subtree_len + 1
            }
        };
        subtree_len
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        if self.nodes.is_empty() {
            return None;
        }

        let inverse_dir = 1.0 / ray.dir;
        let dir_is_neg = [ray.dir.x < 0.0, ray.dir.y < 0.0, ray.dir.z < 0.0];

        let mut nodes_to_visit = ArrayVec::<[usize; 64]>::new();  // used as a stack
        let mut current_node_index = 0;

        let mut interaction = None;

        loop {
            let node = self.nodes[current_node_index];

            if node.bounds.intersect_test(ray).is_some() {
                match node.kind {
                    LinearNodeKind::Leaf {first_prim_idx, n_prims} => {
                        for i in 0..n_prims as usize {
                            let prim = &self.prims[first_prim_idx as usize + i];
                            // sets the variable to be the new (closer, because of the ray t value)
                            // interaction if intersect is Some, or keeps the current interaction
                            // if intersect returns None.
                            interaction = prim.as_ref().intersect(ray).or(interaction);
                        }

                        if let Some(next_node) = nodes_to_visit.pop() {
                            current_node_index = next_node;
                        } else {
                            break;
                        }
                    },

                    LinearNodeKind::Interior {second_child_idx, split_axis} => {
                        if dir_is_neg[split_axis as usize] {
                            nodes_to_visit.push(current_node_index + 1);  // unchecked?
                            current_node_index = second_child_idx as usize;
                        } else {
                            nodes_to_visit.push(second_child_idx as usize);
                            current_node_index += 1;
                        }
                    }
                }
            } else {
                // no intersection with bounding box
                if let Some(next_node) = nodes_to_visit.pop() {
                    current_node_index = next_node;
                } else {
                    break;
                }
            }
        }

        interaction
    }

    pub fn intersect_test(&self, ray: &Ray) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        let inverse_dir = 1.0 / ray.dir;
        let dir_is_neg = [ray.dir.x < 0.0, ray.dir.y < 0.0, ray.dir.z < 0.0];

        let mut nodes_to_visit = ArrayVec::<[usize; 64]>::new();  // used as a stack
        let mut current_node_index = 0;

        loop {
            let node = self.nodes[current_node_index];

            if node.bounds.intersect_test(ray).is_some() {
                match node.kind {
                    LinearNodeKind::Leaf {first_prim_idx, n_prims} => {
                        for i in 0..n_prims as usize {
                            let prim = &self.prims[first_prim_idx as usize + i];
                            if prim.as_ref().intersect_test(ray) { return true; }
                        }

                        if let Some(next_node) = nodes_to_visit.pop() {
                            current_node_index = next_node;
                        } else {
                            break;
                        }
                    },

                    LinearNodeKind::Interior {second_child_idx, split_axis} => {
                        if dir_is_neg[split_axis as usize] {
                            nodes_to_visit.push(current_node_index + 1);  // unchecked?
                            current_node_index = second_child_idx as usize;
                        } else {
                            nodes_to_visit.push(second_child_idx as usize);
                            current_node_index += 1;
                        }
                    }
                }
            } else {
                // no intersection with bounding box
                if let Some(next_node) = nodes_to_visit.pop() {
                    current_node_index = next_node;
                } else {
                    break;
                }
            }
        }

        false
    }
}

// Should be 32 bytes
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LinearBVHNode {
    bounds: Bounds3f,
    kind: LinearNodeKind
}

impl LinearBVHNode {
    fn new_leaf(bounds: Bounds3f, first_prim_idx: u32, n_prims: u16) -> Self {
        Self {
            bounds,
            kind: LinearNodeKind::Leaf { first_prim_idx, n_prims }
        }
    }
    
    fn new_interior(bounds: Bounds3f, second_child_idx: u32, split_axis: u8) -> Self {
        Self {
            bounds,
            kind: LinearNodeKind::Interior { second_child_idx, split_axis }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum LinearNodeKind {
    Leaf {
        first_prim_idx: u32,
        n_prims: u16
    },
    Interior {
        second_child_idx: u32,
        split_axis: u8
    }
}

struct BVHPrimInfo {
    prim_id: usize,
    bounds: Bounds3f,
    centroid: Point3f
}

impl BVHPrimInfo {
    fn new(prim_id: usize, bounds: Bounds3f) -> Self {
        Self { prim_id, bounds, centroid: bounds.centroid() }
    }
}

enum BVHBuildNode<'a> {
    Leaf {
        bounds: Bounds3f,
        first_prim_idx: u32,
        n_prims: u16,
    },

    Interior {
        bounds: Bounds3f,
        children: [&'a BVHBuildNode<'a>; 2],
        split_axis: u8
    }
}

impl<'a> BVHBuildNode<'a> {
    fn new_leaf(first_prim_idx: u32, n_prims: u16, bounds: Bounds3f) -> Self {
        BVHBuildNode::Leaf {
            first_prim_idx, n_prims, bounds
        }
    }

    fn new_interior(children: [&'a BVHBuildNode<'a>; 2], split_axis: u8) -> Self {
        let bounds = children[0].bounds().join(&children[1].bounds());
        BVHBuildNode::Interior {
            children,
            bounds,
            split_axis
        }
    }

    fn bounds(&self) -> Bounds3f {
        match self {
            BVHBuildNode::Leaf {bounds, ..} => *bounds,
            BVHBuildNode::Interior {bounds, ..} => *bounds
        }
    }

}

fn apply_permutation<T>(items: &mut [T], indices: &mut [isize]) {
    // https://stackoverflow.com/a/27507869
    assert_eq!(items.len(), indices.len());

    for i in 0..items.len() {
        if indices[i] < 0 { continue; }

        let mut pos = i;

        while indices[pos] != i as isize {
            let target = indices[pos] as usize;
            items.swap(pos, target);
            indices[pos] = -1 - indices[pos];

            pos = target;
        }

        indices[pos] = -1 - indices[pos];
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{vec3, Vector3};
    use pretty_assertions as pa;
    use rand::{Rng, thread_rng};
    use rand::distributions::{Standard, Uniform, UnitSphereSurface};
    use rand::prelude::*;

    use crate::{Transform, Vec3f};
    use crate::material::Material;
    use crate::primitive::GeometricPrimitive;
    use crate::shapes::sphere::Sphere;

    use super::*;
    use crate::light::AreaLight;

    #[test]
    fn test_permutation() {
        let mut items = vec!["a", "b", "c", "d", "e"];
        let mut perm = vec![2, 3, 0, 1, 4];

        apply_permutation(&mut items, &mut perm);

        assert_eq!(items, vec!["c", "d", "a", "b", "e"])
    }

    #[derive(Copy, Clone)]
    struct MockPrim(Bounds3f);

    impl Primitive for MockPrim {
        fn world_bound(&self) -> Bounds3f {
            self.0
        }

        fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
            unimplemented!()
        }

        fn intersect_test(&self, ray: &Ray) -> bool {
            unimplemented!()
        }
        fn material(&self) -> Option<&dyn Material> { unimplemented!() }

        fn area_light(&self) -> Option<&dyn AreaLight> {
            unimplemented!()
        }
    }

    #[test]
    fn test_bvh() {
        let prim1 = MockPrim(Bounds3f::with_bounds(Point3f::new(1.0, 1.0, 1.0), Point3f::new(2.0, 2.0, 2.0)));
        let prim2 = MockPrim(Bounds3f::with_bounds(Point3f::new(1.0, -1.0, 1.0), Point3f::new(2.0, -2.0, 2.0)));

        let prims: Vec<&dyn Primitive> = vec![&prim1, &prim2];

        let bvh = BVH::build(prims);

        let node1 = LinearBVHNode::new_interior(
            prim1.0.join(&prim2.0),
            2,
            1 // y
        );

        let node2 = LinearBVHNode {
            bounds: prim2.0,
            kind: LinearNodeKind::Leaf {
                first_prim_idx: 0,
                n_prims: 1
            }
        };

        let node3 = LinearBVHNode {
            bounds: prim1.0,
            kind: LinearNodeKind::Leaf {
                first_prim_idx: 1,
                n_prims: 1
            }
        };

        let expected_tree = vec![node1, node2, node3];

        pa::assert_eq!(bvh.nodes, expected_tree);
    }

    #[test]
    fn test_bvh_intersect() {
        let o2w = Transform::translate(vec3(5.0, 5.0, 5.0));
        let sphere1 = Sphere::whole(&o2w, &o2w.inverse(), 1.0);

        let o2w = Transform::translate(vec3(5.0, 5.0, -5.0));
        let sphere2 = Sphere::whole(&o2w, &o2w.inverse(), 1.0);

        let o2w = Transform::translate(vec3(5.0, -5.0, -5.0));
        let sphere3 = Sphere::whole(&o2w, &o2w.inverse(), 1.0);

        let o2w = Transform::translate(vec3(-5.0, -5.0, -5.0));
        let sphere4 = Sphere::whole(&o2w, &o2w.inverse(), 1.0);
    }

    #[test]
    fn test_bvh_intersect_many_nodes() {
        let mut rng = StdRng::from_seed([3; 32]);
        let distr = Uniform::new_inclusive(-10.0, 10.0);
        let tfs: Vec<(Transform, Transform)> = (0..100)
            .map(|_| {
                let v = Vec3f::new(rng.sample(distr), rng.sample(distr), rng.sample(distr));
                let o2w = Transform::translate(v);
                (o2w, o2w.inverse())
            })
            .collect();

        let prims: Vec<GeometricPrimitive<Sphere>> = tfs.iter()
            .map(|(o2w, w2o)| {
                let sphere = Sphere::whole(o2w, w2o, rng.gen_range(0.5, 3.0));
                GeometricPrimitive { shape: sphere, material: None, light: None }
            })
            .collect();

        let mut prim_refs: Vec<&dyn Primitive> = vec![];
        for p in &prims {
            prim_refs.push(p);
        }

        let bvh = BVH::build(prim_refs.clone());

        let mut sphere_surf = UnitSphereSurface::new();
        for i in 0..500 {
            let dir = sphere_surf.sample(&mut rng);
            let dir: Vec3f = Vector3::from(dir).cast().unwrap();
            let mut ray = Ray::new((0.0, 0.0, 0.0).into(), dir);

            let mut bvh_ray = ray.clone();
            let bvh_isect_test = bvh.intersect_test(&bvh_ray);
            let bvh_isect = bvh.intersect(&mut bvh_ray);

            let expected_test = intersect_test_list(&ray, &prim_refs);
            let expected_isect = intersect_list(&mut ray, &prim_refs);

            assert_eq!(expected_test, expected_isect.is_some(), "Iteration {}", i);
            assert_eq!(bvh_isect_test, bvh_isect.is_some(), "Iteration {}", i);
            assert_eq!(bvh_isect.map(|i| i.hit), expected_isect.map(|i| i.hit), "Iteration {}", i);
            assert_eq!(bvh_isect_test, expected_test, "Iteration {}", i);
        }
    }

    fn intersect_test_list(ray: &Ray, prims: &[&dyn Primitive]) -> bool {
        prims.iter().any(|prim| {
            prim.intersect_test(ray)
        })
    }

    fn intersect_list<'p>(ray: &mut Ray, prims: &'p [&dyn Primitive]) -> Option<SurfaceInteraction<'p>> {
        let mut isect = None;
        for prim in prims {
            isect = prim.intersect(ray).or(isect);
        }
        isect
    }
}