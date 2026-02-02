use crate::vectors::Vec3;


pub struct Triangle {
    pub indices: [usize; 3],
    pub normals: [Vec3; 3],
    pub color: Vec3,
    pub emission_strength: f32,
    pub emission_color: Vec3,
}

pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<Triangle>,
    pub scale: Vec3,
    pub position: Vec3,
    pub rotation: Vec3,
}


pub struct World {
    pub meshes: Vec<Mesh>,
}


pub struct RayHit {
    pub color: Vec3,
    pub distance: f32,
    pub position: Vec3,
    pub direction: Vec3,
    pub reflected_dir: Vec3,
}

impl Default for RayHit {
    fn default() -> RayHit {
        RayHit {
            color: Vec3::zero(),
            distance: 0.0,
            position: Vec3::zero(),
            direction: Vec3::zero(),
            reflected_dir: Vec3::zero(),
        }
    }
}