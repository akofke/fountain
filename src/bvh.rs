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

}

impl BVH {
    pub fn build(prims: Vec<Rc<dyn Primitive>>) -> BVH {
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



        unimplemented!()
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
            let node = arena.alloc(BVHBuildNode::new_leaf(first_prim_idx, n_prims, node_bounds));
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

        arena.alloc(BVHBuildNode::new_interior([child1, child2], ax))
    }
}

// Should be 32 bytes
pub enum LinearBVHNode {
    Leaf {
        bounds: Aabb,
        primitives_offset: u32,
        n_primitives: u16
    },
    Interior {
        bounds: Aabb,
        second_child_offset: u32,
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
        first_prim_idx: usize,
        n_prims: usize,
    },

    Interior {
        bounds: Aabb,
        children: [&'a BVHBuildNode<'a>; 2],
        split_axis: usize
    }
}

impl<'a> BVHBuildNode<'a> {
    fn new_leaf(first_prim_idx: usize, n_prims: usize, bounds: Aabb) -> Self {
        BVHBuildNode::Leaf {
            first_prim_idx, n_prims, bounds
        }
    }

    fn new_interior(children: [&'a BVHBuildNode<'a>; 2], split_axis: usize) -> Self {
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