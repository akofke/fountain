use crate::primitive::Primitive;
use crate::{Vec3f, Ray, SurfaceInteraction};
use bumpalo::Bump;
use std::ops::{Range, DerefMut};
use partition::partition;
use std::rc::Rc;
use std::fmt::Debug;
use crate::geometry::bounds::Bounds3f;
use crate::Point3f;
use arrayvec::ArrayVec;
use std::sync::Arc;

#[derive(Copy, Clone)]
pub enum SplitMethod {
    Middle,
    EqualCounts,
    SAH
}

pub struct BVH<'p> {
    pub prims: Vec<&'p dyn Primitive>,
    pub bounds: Bounds3f,
    nodes: Vec<LinearBVHNode>
}

impl BVH<'_> {
    pub fn build(mut prims: Vec<&dyn Primitive>) -> BVH {
        // TODO: figure out prims type. Rc or Box?

        if prims.len() == 0 {
            return BVH { prims, bounds: Bounds3f::empty(), nodes: Vec::new() }
        }

        let mut prim_info: Vec<BVHPrimInfo> = prims.iter().enumerate().map(|(i, p)| {
            BVHPrimInfo::new(i, p.world_bound())
        }).collect();

        let arena = Bump::new();
        let mut prim_ordering: Vec<isize> = Vec::with_capacity(prims.len());
        let range = 0..prim_info.len();

        let root = Self::recursive_build(
            &arena,
            &mut prim_info,
//            range,
            &mut prim_ordering,
            SplitMethod::Middle
        );

        let world_bound = root.bounds();

        apply_permutation(&mut prims, &mut prim_ordering);

        let mut flat_nodes = Vec::<LinearBVHNode>::with_capacity(prims.len());

        Self::flatten_tree(&mut flat_nodes, root, 0);

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
                partition(prim_info, |prim| {
                    prim.centroid[ax] < midpoint
                })

            },
            _ => unimplemented!()
        };

        let child1 = Self::recursive_build(arena, part1, prim_ordering, split_method);
        let child2 = Self::recursive_build(arena, part2, prim_ordering, split_method);

        arena.alloc(BVHBuildNode::new_interior([child1, child2], ax as u8))
    }

    // Returns subtree length
    fn flatten_tree(flat_nodes: &mut Vec<LinearBVHNode>, node: &BVHBuildNode, idx: usize) -> usize {
        let subtree_len = match node {
            &BVHBuildNode::Leaf {bounds, first_prim_idx, n_prims} => {
                let leaf = LinearBVHNode::new_leaf(bounds, first_prim_idx, n_prims);
                flat_nodes.push(leaf);
                1
            },

            &BVHBuildNode::Interior {bounds, children, split_axis} => {
                let interior = LinearBVHNode::new_interior(bounds, 0, split_axis);
                flat_nodes.push(interior);
                let first_subtree_len = Self::flatten_tree(flat_nodes, children[0], idx + 1);
                let second_idx = idx + first_subtree_len + 1;
                if let LinearNodeKind::Interior {ref mut second_child_idx, ..} = flat_nodes[idx].kind {
                    *second_child_idx = second_idx as u32;
                } else { unreachable!() } // unchecked?

                first_subtree_len + Self::flatten_tree(flat_nodes, children[1], second_idx)
            }
        };
        subtree_len
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        if self.nodes.len() == 0 {
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
                            interaction = prim.intersect(ray);
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
        if self.nodes.len() == 0 {
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
                            if prim.intersect_test(ray) { return true; }
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
    use super::*;
    use pretty_assertions as pa;
    use crate::material::Material;

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
}