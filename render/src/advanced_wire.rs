use std::collections::HashMap;

use glam::Vec3;
use ordered_float::OrderedFloat;

#[derive(Clone)]
struct Vertex {
    pub pos: Vec3,
    pub abs_index: usize,
}

impl Vertex {
    pub fn new(abs_index: usize, pos: Vec3) -> Self {
        Self { abs_index, pos }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct OrderedVec3 {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
    pub z: OrderedFloat<f32>,
}

impl OrderedVec3 {
    pub fn new(x: OrderedFloat<f32>, y: OrderedFloat<f32>, z: OrderedFloat<f32>) -> OrderedVec3 {
        Self { x, y, z }
    }
}

impl OrderedVec3 {
    pub fn eq_vec3(&self, v: &Vec3) -> bool {
        self.x == v.x && self.y == v.y && self.z == v.z
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Edge {
    pub v1: OrderedVec3,
    pub v2: OrderedVec3,
}

impl Edge {
    pub fn new(v1: Vec3, v2: Vec3) -> Edge {
        Self {
            v1: OrderedVec3::new(OrderedFloat(v1.x), OrderedFloat(v1.y), OrderedFloat(v1.z)),
            v2: OrderedVec3::new(OrderedFloat(v2.x), OrderedFloat(v2.y), OrderedFloat(v2.z)),
        }
    }

    pub fn invert(self) -> Self {
        let mut new = self.clone();
        std::mem::swap(&mut new.v1, &mut new.v2);
        new
    }
}

#[derive(Clone)]
struct Triangle {
    pub v1: Vertex,
    pub v2: Vertex,
    pub v3: Vertex,
}

type EdgeCommons = HashMap<Edge, (Triangle, Option<Triangle>)>;

fn _add_ec(commons: &mut EdgeCommons, tri: &Triangle, edge: Edge) {
    let opt1 = commons.get_mut(&edge);
    if let Some(x) = opt1 {
        x.1 = Some(tri.clone());
        return;
    }
    let edge = edge.invert();
    if let Some(x) = commons.get_mut(&edge) {
        x.1 = Some(tri.clone());
        return;
    }
    commons.insert(edge, (tri.clone(), None));
}

fn add_edge_common(commons: &mut EdgeCommons, tri: &Triangle) {
    let e1 = Edge::new(tri.v1.pos, tri.v2.pos);
    let e2 = Edge::new(tri.v2.pos, tri.v3.pos);
    let e3 = Edge::new(tri.v3.pos, tri.v1.pos);

    assert_ne!(e1, e2);
    assert_ne!(e2, e3);
    assert_ne!(e3, e1);

    _add_ec(commons, tri, e1);
    _add_ec(commons, tri, e2);
    _add_ec(commons, tri, e3);
}

fn hide_edge(barys: &mut [f32], edge: &Edge, tri1: &Triangle) {
    let v1_index = if edge.v1.eq_vec3(&tri1.v1.pos) {
        tri1.v1.abs_index
    } else if edge.v1.eq_vec3(&tri1.v2.pos) {
        tri1.v2.abs_index
    } else {
        tri1.v3.abs_index
    };
    let v2_index = if edge.v2.eq_vec3(&tri1.v1.pos) {
        tri1.v1.abs_index
    } else if edge.v2.eq_vec3(&tri1.v2.pos) {
        tri1.v2.abs_index
    } else {
        tri1.v3.abs_index
    };

    let bary_v1 = Vec3::new(barys[v1_index], barys[v1_index + 1], barys[v1_index + 2]);
    let bary_v2 = Vec3::new(barys[v2_index], barys[v2_index + 1], barys[v2_index + 2]);
    let bary_v3 = Vec3::ONE - bary_v1 - bary_v2;
    barys[v1_index] += bary_v3[0];
    barys[v1_index + 1] += bary_v3[1];
    barys[v1_index + 2] += bary_v3[2];
    barys[v2_index] += bary_v3[0];
    barys[v2_index + 1] += bary_v3[1];
    barys[v2_index + 2] += bary_v3[2];
}

pub fn calculate_barycentrics_adv(triangles: &[f32], normals: &[f32]) -> Vec<f32> {
    let mut barys: Vec<f32> =
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0].repeat(triangles.len() / 9);
    let mut edge_commons: EdgeCommons = HashMap::new();

    let mut i = 0;
    while i <= triangles.len() - 9 {
        let a = Vec3::new(triangles[i], triangles[i + 1], triangles[i + 2]);
        let b = Vec3::new(triangles[i + 3], triangles[i + 4], triangles[i + 5]);
        let c = Vec3::new(triangles[i + 6], triangles[i + 7], triangles[i + 8]);

        let v1 = Vertex::new(i, a);
        let v2 = Vertex::new(i + 3, b);
        let v3 = Vertex::new(i + 6, c);

        let tri = Triangle { v1, v2, v3 };

        add_edge_common(&mut edge_commons, &tri);

        i += 9;
    }

    for (k, v) in edge_commons {
        if v.1.is_none() {
            continue;
        }
        let tri1 = v.0;
        let tri2 = v.1.unwrap();

        let normal1 = Vec3::new(
            normals[tri1.v1.abs_index],
            normals[tri1.v1.abs_index + 1],
            normals[tri1.v1.abs_index + 2],
        );
        let normal2 = Vec3::new(
            normals[tri2.v1.abs_index],
            normals[tri2.v1.abs_index + 1],
            normals[tri2.v1.abs_index + 2],
        );

        let diff = (normal1 - normal2).abs();
        if diff.x < 0.01 && diff.y < 0.01 && diff.z < 0.01 {
            hide_edge(&mut barys, &k, &tri1);
            hide_edge(&mut barys, &k, &tri2);
        }
    }

    barys
}

pub fn calculate_barycentrics_quad(triangles: &[f32]) -> Vec<f32> {
    let mut barys: Vec<f32> =
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0].repeat(triangles.len() / 9);

    let mut i = 0;
    while i <= triangles.len() - 9 {
        let a = Vec3::new(triangles[i], triangles[i + 1], triangles[i + 2]);
        let b = Vec3::new(triangles[i + 3], triangles[i + 4], triangles[i + 5]);
        let c = Vec3::new(triangles[i + 6], triangles[i + 7], triangles[i + 8]);

        let e1 = b.distance_squared(a);
        let e2 = b.distance_squared(c);
        let e3 = c.distance_squared(a);

        if e1 > e2 && e1 > e3 {
            hide_edge_quad(&mut barys, i + 6, i, i + 3)
        } else if e2 > e1 && e2 > e3 {
            hide_edge_quad(&mut barys, i, i + 3, i + 6)
        } else if e3 > e1 && e3 > e2 {
            hide_edge_quad(&mut barys, i + 3, i, i + 6)
        }
        i += 9;
    }

    barys
}

fn hide_edge_quad(barys: &mut [f32], color_index: usize, edge_point_a: usize, edge_point_b: usize) {
    let b_color = Vec3::new(
        barys[color_index],
        barys[color_index + 1],
        barys[color_index + 2],
    );
    barys[edge_point_a] += b_color.x;
    barys[edge_point_a + 1] += b_color.y;
    barys[edge_point_a + 2] += b_color.z;

    //println!("Edge 1: ({},{},{})", barys[edge_point_a], barys[edge_point_a+1], barys[edge_point_a+2]);

    barys[edge_point_b] += b_color.x;
    barys[edge_point_b + 1] += b_color.y;
    barys[edge_point_b + 2] += b_color.z;

    //println!("Edge 2: ({},{},{})", barys[edge_point_b], barys[edge_point_b+1], barys[edge_point_b+2]);
}
