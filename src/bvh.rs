use crate::primitive::Primitive;
use crate::aabb::{Aabb, Axis};
use crate::Vec3;
use bumpalo::Bump;
use std::ops::Range;


pub struct BVH {

}

impl BVH {
    pub fn build(prims: Vec<&dyn Primitive>) -> BVH {

        let mut prim_info: Vec<BVHPrimInfo> = prims.iter().enumerate().map(|(i, p)| {
            BVHPrimInfo::new(i, p.world_bound())
        }).collect();

        let arena = Bump::new();
        let mut prim_ordering: Vec<usize> = Vec::with_capacity(prims.len());
        let range = 0..prim_info.len();

        let root = Self::recursive_build(
            &arena,
            &mut prim_info,
//            range,
            &mut prim_ordering
        );

        unimplemented!()
    }

    fn recursive_build<'a>(
        arena: &'a Bump,
        prim_info: &mut [BVHPrimInfo],
//        range: Range<usize>,
        prim_ordering: &mut Vec<usize>
    ) -> &'a BVHBuildNode<'a> {

        let node_bounds = prim_info.iter()
            .fold(Aabb::empty(), |node_bb, prim| { node_bb.join(&prim.bounds)});

        let n_prims = prim_info.len();

        if n_prims == 1 {
            // Create leaf node with single primitive
            let first_prim_idx = prim_ordering.len();
            for prim in prim_info {
                prim_ordering.push(prim.prim_id)
            }
            let node = arena.alloc(BVHBuildNode::new_leaf(first_prim_idx, n_prims, node_bounds));
            return node;
        }

        let centroid_bounds = prim_info.iter()
            .fold(Aabb::empty(), |centroid_bb, prim| { centroid_bb.join_point(&prim.centroid)});

        let ax = centroid_bounds.maximum_extent();

        let mid = prim_info.len() / 2;
        unimplemented!()
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
        split_axis: Axis
    }
}

impl<'a> BVHBuildNode<'a> {
    fn new_leaf(first_prim_idx: usize, n_prims: usize, bounds: Aabb) -> Self {
        BVHBuildNode::Leaf {
            first_prim_idx, n_prims, bounds
        }
    }

    fn new_interior(children: [&'a BVHBuildNode<'a>; 2], split_axis: Axis) -> Self {
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