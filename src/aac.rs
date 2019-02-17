//! BVH implementation using Approximate Agglomerative Clustering

use crate::aabb::Aabb;
use crate::scene::PrimRef;
use crate::morton::morton3;

struct Aac {

}

enum Cluster {
    Leaf {
    },
    Inner {

    }
}

impl Aac {
    pub fn build(primitives: &mut [PrimRef]) -> Self {

        unimplemented!()
    }

    fn sort_primitives(primitives: &mut [PrimRef]) {
        // find scene bounding box
        let scene_bb = primitives.iter().fold(Aabb::empty(), |scene_bb, prim| scene_bb.join(&prim.aabb));

        // TODO: radix sort!!
        primitives.sort_by_cached_key(|prim| {
            let centroid = prim.aabb.normalized_by(&scene_bb).centroid();
            morton3(centroid.x, centroid.y, centroid.z)
        })
    }
}