//! BVH implementation using Approximate Agglomerative Clustering

use crate::aabb::Aabb;

struct Aac {

}

enum Cluster {
    Leaf {
    },
    Inner {

    }
}

impl Aac {
    pub fn build(primitives: &mut [Aabb]) -> Self {

        unimplemented!()
    }

    fn sort_primitives(primitives: &mut [Aabb]) {
        // find scene bounding box
        let scene_bb = primitives.iter().fold(Aabb::empty(), |scene_bb, prim_bb| scene_bb.grow(prim_bb));
    }
}