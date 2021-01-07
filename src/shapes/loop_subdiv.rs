use crate::{Transform, Point3f};
use crate::shapes::triangle::TriangleMesh;
use bumpalo::Bump;
use crate::id_arena::{Id, IdArena};

struct SDVertex {
    p: Point3f,
    start_face: Option<Id<SDFace>>,
    child: Option<Id<SDVertex>>,
    regular: bool,
    boundary: bool,
}

impl SDVertex {
    pub fn new(p: Point3f) -> Self {
        Self {
            p,
            start_face: None,
            child: None,
            regular: false,
            boundary: false,
        }
    }
}

struct SDFace {
    v: [Id<SDVertex>; 3],
    f: [Option<Id<SDFace>>; 3],
    children: [Option<Id<SDFace>>; 4]

}

pub fn loop_subdivide(
    obj_to_world: &Transform,
    n_levels: u32,
    vertices: &[Point3f],
    vertex_indices: &[u32],
) -> TriangleMesh {

    assert_eq!(vertex_indices.len() % 3, 0);
    let n_faces = vertex_indices.len() / 3;

    let mut verts = IdArena::new();
    let mut faces = IdArena::new();

    let initial_verts = vertices
        .iter()
        .map(|p| verts.insert(SDVertex::new(*p)))
        .collect::<Vec<_>>();

    let (face_indices, _) = vertex_indices.as_chunks::<3>();
    let initial_faces = vertex_indices
        .array_chunks::<3>()
        .map(|[v1, v2, v3]| {
            let v = [
                initial_verts[v1 as usize],
                initial_verts[v2 as usize],
                initial_verts[v3 as usize],
            ];
            faces.insert(SDFace {
                v,
                f: [None; 3],
                children: [None; 4]
            })
        })

    // let initial_faces = arena.alloc_slice_fill_with(n_faces, |i_face| {
    //     let [v1, v2, v3] = face_indices[i_face];
    //     let face = SDFace {
    //         v: [&initial_verts[v1 as usize], &initial_verts[v2 as usize], &initial_verts[v3 as usize]],
    //         f: [None; 3],
    //         children: [None; 4],
    //     };
    //     face
    // });
    //
    // for (verts, face) in initial_verts.array_chunks_mut::<3>().zip(initial_faces) {
    //     for v in verts.iter_mut() {
    //         v.start_face = Some(face);
    //     }
    // }



    unimplemented!()
}