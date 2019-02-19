//! BVH implementation using Approximate Agglomerative Clustering

use crate::aabb::Aabb;
use crate::scene::PrimRef;
use crate::morton::morton3;

struct Aac {
    morton_codes: Vec<u32>,
}

enum Cluster {
    Leaf {
    },
    Inner {

    }
}

impl Aac {
    pub fn build(primitives: &mut [PrimRef]) -> Self {

        // find scene bounding box
        let scene_bb = primitives.iter().fold(Aabb::empty(), |scene_bb, prim| scene_bb.join(&prim.aabb));

        let mut morton_codes = primitives.iter().map(|prim| {
            let centroid = prim.aabb.normalized_by(&scene_bb).centroid();
            morton3(centroid.x, centroid.y, centroid.z)
        }).collect::<Vec<_>>();

        Aac::sort_primitives(primitives, &mut morton_codes);

        unimplemented!()
    }

    fn sort_primitives(primitives: &mut [PrimRef], morton_codes: &mut [u32]) {
        // TODO: radix sort!!
        primitives.sort_unstable_by_key(|prim| morton_codes[prim.idx]);
        morton_codes.sort_unstable();
    }

}