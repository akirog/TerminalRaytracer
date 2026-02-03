use crate::vectors::Vec3;


pub struct Triangle {
    pub indices: [usize; 3],
    pub normals: [Vec3; 3],
}


#[derive(Copy, Clone)]
pub struct Material {
    /// A materials ability to reflect light
    pub albedo: Vec3,

    /// How much light the material emits
    pub emission_strength: f32,

    /// The color of emitted light
    pub emission_color: Vec3,

    /// How much reflected rays are scattered
    pub roughness: f32,
}

impl Default for Material {
    fn default() -> Material {
        Material {
            albedo: Vec3::new(1.0, 1.0, 1.0),
            emission_color: Vec3::default(),
            emission_strength: 0.0,
            roughness: 0.0,
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<Triangle>,
    pub scale: Vec3,
    pub position: Vec3,
    pub rotation: Vec3,
    pub material: Material,
}


pub struct World {
    pub meshes: Vec<Mesh>,
}


pub struct RayHit {
    pub material: Material,
    pub distance: f32,
    pub position: Vec3,
    pub direction: Vec3,
    pub reflected_dir: Vec3,
}

impl Default for RayHit {
    fn default() -> RayHit {
        RayHit {
            material: Material::default(),
            distance: 0.0,
            position: Vec3::zero(),
            direction: Vec3::zero(),
            reflected_dir: Vec3::zero(),
        }
    }
}
