use crate::primitive::Primitive;
use crate::aabb::{Aabb, Axis};
use crate::Vec3;
use bumpalo::Bump;
use std::ops::{Range, DerefMut};
use partition::partition;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub enum SplitMethod {
    Middle,
    EqualCounts,
    SAH
}

pub struct BVH {
    pub prims: Vec<Rc<dyn Primitive>>,
    nodes: Vec<LinearBVHNode>
}

impl BVH {
    pub fn build(mut prims: Vec<Rc<dyn Primitive>>) -> BVH {
        // TODO: figure out prims type. Rc or Box?

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

        apply_permutation(&mut prims, &mut prim_ordering);

        let mut flat_nodes = Vec::<LinearBVHNode>::with_capacity(prims.len());

        Self::flatten_tree(&mut flat_nodes, root, 0);

        BVH {
            prims,
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
            .fold((Aabb::empty(), Aabb::empty()), |(node_bb, centr_bb), prim| {
                (node_bb.join(&prim.bounds), centr_bb.join_point(&prim.centroid))
            });

        let n_prims = prim_info.len();

        // If there is only one primitive or all the centroids lie on the same point
        // (and therefore can't be partitioned), create a leaf node.
        if n_prims == 1 || centroid_bounds.is_point() {
            let first_prim_idx = prim_ordering.len();
            for prim in prim_info {
                prim_ordering.push(prim.prim_id as isize)
            }
            let node = arena.alloc(BVHBuildNode::new_leaf(first_prim_idx as u32, n_prims as u16, node_bounds));
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
        let child2 = Self::recursive_build(arena, part1, prim_ordering, split_method);

        arena.alloc(BVHBuildNode::new_interior([child1, child2], ax as u8))
    }

    // Returns subtree length
    fn flatten_tree(flat_nodes: &mut Vec<LinearBVHNode>, node: &BVHBuildNode, idx: usize) -> usize {
        let subtree_len = match node {
            &BVHBuildNode::Leaf {bounds, first_prim_idx, n_prims} => {
                let leaf = LinearBVHNode::Leaf {bounds, first_prim_idx, n_prims};
                flat_nodes.push(leaf);
                1
            },

            &BVHBuildNode::Interior {bounds, children, split_axis} => {
                let interior = LinearBVHNode::Interior {bounds, split_axis, second_child_idx: 0};
                flat_nodes.push(interior);
                let first_subtree_len = Self::flatten_tree(flat_nodes, children[0], idx + 1);
                let second_idx = idx + first_subtree_len;
                if let LinearBVHNode::Interior {ref mut second_child_idx, ..} = flat_nodes[idx] {
                    *second_child_idx = second_idx as u32;
                } else { unreachable!() } // unchecked?

                first_subtree_len + Self::flatten_tree(flat_nodes, children[1], second_idx)
            }
        };
        subtree_len
    }
}

// Should be 32 bytes
#[derive(Copy, Clone)]
pub enum LinearBVHNode {
    Leaf {
        bounds: Aabb,
        first_prim_idx: u32,
        n_prims: u16
    },
    Interior {
        bounds: Aabb,
        second_child_idx: u32,
        split_axis: u8
    }
}

struct BVHPrimInfo {
    prim_id: usize,
    bounds: Aabb,
    centroid: Vec3
}

impl BVHPrimInfo {
    fn new(prim_id: usize, bounds: Aabb) -> Self {
        Self { prim_id, bounds, centroid: bounds.centroid() }
    }
}

enum BVHBuildNode<'a> {
    Leaf {
        bounds: Aabb,
        first_prim_idx: u32,
        n_prims: u16,
    },

    Interior {
        bounds: Aabb,
        children: [&'a BVHBuildNode<'a>; 2],
        split_axis: u8
    }
}

impl<'a> BVHBuildNode<'a> {
    fn new_leaf(first_prim_idx: u32, n_prims: u16, bounds: Aabb) -> Self {
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

    fn bounds(&self) -> Aabb {
        match self {
            BVHBuildNode::Leaf {bounds, ..} => *bounds,
            BVHBuildNode::Interior {bounds, ..} => *bounds
        }
    }

}

fn apply_permutation<T>(items: &mut [T], indices: &mut [isize]) {
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

    #[test]
    fn test_permutation() {
        let mut items = vec!["a", "b", "c", "d", "e"];
        let mut perm = vec![2, 3, 0, 1, 4];

        apply_permutation(&mut items, &mut perm);

        assert_eq!(items, vec!["c", "d", "a", "b", "e"])
    }
}