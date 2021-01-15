use crate::{Transform, Point3f};
use crate::shapes::triangle::TriangleMesh;
use bumpalo::Bump;
use crate::id_arena::{Id, IdArena};
use std::collections::HashMap;
use cgmath::Array;
use arrayvec::ArrayVec;

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
    v: Option<[Id<SDVertex>; 3]>,
    f: [Option<Id<SDFace>>; 3],
    children: Option<[Id<SDFace>; 4]>,

}

impl SDFace {
    pub fn vert_num(&self, vert: Id<SDVertex>) -> usize {
        self.v.unwrap().iter()
            .enumerate()
            .find(|(_, &item)| item == vert)
            .expect("Tried to find index of vertex not in this face")
            .0
    }

    pub fn next_face(&self, vert: Id<SDVertex>) -> Option<Id<SDFace>> {
        self.f[self.vert_num(vert)]
    }

    pub fn prev_face(&self, vert: Id<SDVertex>) -> Option<Id<SDFace>> {
        self.f[prev(self.vert_num(vert))]
    }

    pub fn next_vert(&self, vert: Id<SDVertex>) -> Id<SDVertex> {
        self.v.unwrap()[next(self.vert_num(vert))]
    }

    pub fn prev_vert(&self, vert: Id<SDVertex>) -> Id<SDVertex> {
        self.v.unwrap()[prev(self.vert_num(vert))]
    }
}

#[derive(Eq, PartialEq, Hash)]
struct SDEdge {
    v: [Id<SDVertex>; 2],
}

impl SDEdge {
    pub fn new(v1: Id<SDVertex>, v2: Id<SDVertex>) -> Self {
        // store in a canonical order, so that edge v1->v2 is treated as the same as v2->v1
        Self {
            v: [v1.min(v2), v1.max(v2)]
        }
    }
}

struct SDEdgeData {
    adjacent_face: Id<SDFace>,
    face_edge_num: u8,
}

fn next(i: usize) -> usize {
    (i + 1) % 3
}

fn prev(i: usize) -> usize {
    (i + 2) % 3
}

struct SDData {
    vertices: IdArena<SDVertex>,
    faces: IdArena<SDFace>,
}

impl SDData {
    pub fn iter_adjacent_faces_forward(&self, vert_id: Id<SDVertex>) -> impl Iterator<Item = Id<SDFace>> + '_ {
        let start_face = self.vertices.get(vert_id).start_face.unwrap();
        std::iter::successors(Some(start_face), move |&face| {
            self.faces.get(face).next_face(vert_id)
        })
    }

    pub fn iter_adjacent_faces_backward(&self, vert_id: Id<SDVertex>) -> impl Iterator<Item = Id<SDFace>> + '_ {
        let start_face = self.vertices.get(vert_id).start_face.unwrap();
        std::iter::successors(Some(start_face), move |&face| {
            self.faces.get(face).prev_face(vert_id)
        })
    }

    pub fn vertex_valence(&self, vert_id: Id<SDVertex>) -> usize {
        let boundary = self.vertices.get(vert_id).boundary;
        let start_face = self.vertices.get(vert_id).start_face.unwrap();

        if !boundary {
            self.iter_adjacent_faces_forward(vert_id)
                .skip(1)
                .take_while(|&face| face != start_face)
                .count() + 1
        } else {
            // go forward until hitting an edge to count one side, then backwards until hitting an
            // edge to count the other side. Skip start face on backwards to avoid double counting,
            // but add one because that's how valence is defined for edge vertices.
            let count = self.iter_adjacent_faces_forward(vert_id).count();
            count
                + self.iter_adjacent_faces_backward(vert_id).skip(1).count()
                + 1
        }
    }
}

pub fn loop_subdivide(
    obj_to_world: &Transform,
    n_levels: u32,
    vertices: &[Point3f],
    vertex_indices: &[u32],
) -> TriangleMesh {

    assert_eq!(vertex_indices.len() % 3, 0);
    let n_faces = vertex_indices.len() / 3;

    let mut data = SDData {
        vertices: IdArena::new(),
        faces: IdArena::new(),
    };

    let initial_verts = vertices
        .iter()
        .map(|p| data.vertices.insert(SDVertex::new(*p)))
        .collect::<Vec<_>>();

    let (face_indices, _) = vertex_indices.as_chunks::<3>();
    let initial_faces = vertex_indices
        .array_chunks::<3>()
        .map(|&[v1, v2, v3]| {
            let v = [
                initial_verts[v1 as usize],
                initial_verts[v2 as usize],
                initial_verts[v3 as usize],
            ];
            data.faces.insert(SDFace {
                v: Some(v),
                f: [None; 3],
                children: None,
            })
        })
        .collect::<Vec<_>>();

    for &face_id in &initial_faces {
        let face = data.faces.get(face_id);
        for &vert_id in &face.v.unwrap() {
            data.vertices.get_mut(vert_id).start_face = Some(face_id);
        }
    }

    // Go through all faces and compute adjacent faces.
    // edges map holds all edges seen so far with one face "side" known.
    let mut edges: HashMap<SDEdge, SDEdgeData> = HashMap::new();
    for &face_id in &initial_faces {

        for edge_num in 0..3 {
            let v0 = edge_num;
            let v1 = next(edge_num);
            let face = data.faces.get(face_id);
            let fv = face.v.unwrap();
            let edge = SDEdge::new(fv[v0], fv[v1]);

            if let Some(edge_data) = edges.remove(&edge) {
                // we've seen this edge before on another face, so set that face as adjacent
                let nbr_face = data.faces.get_mut(edge_data.adjacent_face);
                nbr_face.f[edge_data.face_edge_num as usize] = Some(face_id);
                data.faces.get_mut(face_id).f[edge_num] = Some(edge_data.adjacent_face);
            } else {
                // this edge is new so store the edge and our own info in the map
                edges.insert(edge, SDEdgeData {
                    adjacent_face: face_id,
                    face_edge_num: edge_num as u8,
                });
            }
        }
    }

    for &vert_id in &initial_verts {
        let start_face = data.vertices.get(vert_id).start_face.unwrap();

        let boundary = data.iter_adjacent_faces_forward(vert_id)
            .skip(1)
            .any(|face| face == start_face);

        data.vertices.get_mut(vert_id).boundary = boundary;

        let valence = data.vertex_valence(vert_id);
        let regular = if !boundary && valence == 6 {
            true
        } else if boundary && valence == 4 {
            true
        } else {
            false
        };

        data.vertices.get_mut(vert_id).regular = regular;
    }

    let mut f = initial_faces;
    let mut v = initial_verts;
    for _level in 0..n_levels {
        let mut new_faces = Vec::new();
        let mut new_vertices = Vec::new();

        // allocate next level of children in mesh tree
        for &vert_id in &v {
            let child = data.vertices.insert(SDVertex::new(Point3f::from_value(0.0)));
            data.vertices.get_mut(child).regular = data.vertices.get(vert_id).regular;
            data.vertices.get_mut(child).boundary = data.vertices.get_mut(vert_id).boundary;
            data.vertices.get_mut(vert_id).child = Some(child);
            new_vertices.push(child);
        }

        for &face_id in &f {
            let mut children = ArrayVec::<[_; 4]>::new();
            for i in 0..4 {
                children.push(data.faces.insert(SDFace {
                    v: None,
                    f: [None; 3],
                    children: None
                }))
            }
            // TODO clean up arrayvec
            data.faces[face_id].children = Some(children.into_inner().unwrap());
        }
    }

    unimplemented!()
}

