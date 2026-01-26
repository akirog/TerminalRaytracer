use std::ops::*;
use std::process::Output;


#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}


impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn zero() -> Vec3 {
        Vec3 { x: 0.0, y: 0.0, z: 0.0 }
    }
    
    pub fn length(self) -> f32 { (self.x*self.x + self.y*self.y + self.z*self.z).sqrt() }
    
    pub fn normalized(self) -> Vec3 { self / self.length() }
    
    pub fn dot(self, other: Vec3) -> f32 { self.x * other.x + self.y * other.y + self.z * other.z }
    
    pub fn cross(self, other: Vec3) -> Vec3 { 
        Vec3 { 
            x: self.y * other.z - self.z * other.y, 
            y: self.z * other.x - self.x * other.z, 
            z: self.x * other.y - self.y * other.x 
        } 
    }
    
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl AddAssign<f32> for Vec3 {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}




impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl SubAssign<f32> for Vec3 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}


impl Mul<f32> for Vec3 {
    type Output = Vec3;
    
    fn mul(self, rhs: f32) -> Self::Output {
        Vec3 { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}


impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, other: f32) -> Self::Output {
        Vec3 {x: self.x / other, y: self.y / other, z: self.z / other}
    }
}