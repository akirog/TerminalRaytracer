extern crate core;

use macroquad::prelude::*;

use std::sync::{Arc};
use rayon::prelude::*;

pub mod vectors;
mod my3d_lib;

use crate::vectors::Vec3;
use crate::my3d_lib::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Macroquad Raytracer".to_owned(),
        window_width: 700,
        window_height: 700,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Create map
    let world: World  = generate_map();
    let world_arc = Arc::new(world); // Move map into Arc

    // Player state
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let mut yaw: f32 = 0.0;
    let mut pitch: f32 = 0.0;

    let speed: f32 = 2.0;
    let mouse_sensitivity: f32 = 0.2;

    // Hide cursor and capture
    set_cursor_grab(true);
    show_mouse(false);

    loop {
        clear_background(BLACK);



        // --- Input ---
        let delta_pos = mouse_delta_position();
        yaw -= delta_pos.x * mouse_sensitivity;
        pitch -= delta_pos.y * mouse_sensitivity;


        let mut forward: Vec3 = Vec3::zero();
        forward.x = f32::sin(yaw) * f32::cos(pitch);
        forward.y = f32::cos(yaw) * f32::cos(pitch);
        forward.z = f32::sin(pitch);

        let world_up = Vec3::new(0.0, 0.0, 1.0);

        let right = forward.cross(world_up).normalized();
        let up = right.cross(forward).normalized();


        let dt = get_frame_time();
        let amount = speed * dt;

        if is_key_down(KeyCode::W) { pos += forward * amount; }
        if is_key_down(KeyCode::S) { pos -= forward * amount; }

        if is_key_down(KeyCode::D) { pos += right * amount; }
        if is_key_down(KeyCode::A) { pos -= right * amount; }

        if is_key_down(KeyCode::Space)     { pos -= up * amount; }
        if is_key_down(KeyCode::LeftShift) { pos += up * amount; }

        if is_key_down(KeyCode::Escape) { break; }



        // --- Rendering ---
        let width = screen_width() as usize;
        let height = screen_height() as usize;
        let mut image = Image::gen_image_color(width as u16, height as u16, BLACK);

        // Create pixel buffer
        let mut pixels = vec![Color::default(); width * height];


        let inv_width = 1.0 / width as f32;
        let inv_height = 1.0 / height as f32;


        // Parallel iteration over pixels
        pixels.par_iter_mut()
            .enumerate()
            .for_each(|(index, pixel)| {
                let x = index % width;
                let y = index / width;

                // Get from -0.5 to 0.5
                let screen_x = (x as f32 * inv_width - 0.5) * 2.0;
                let screen_y = (y as f32 * inv_height - 0.5) * 2.0;

                let mut dir = forward;
                dir += right * screen_x * 1.0;
                dir += up * screen_y * 1.0;
                dir = dir.normalized();

                let color = cast_ray_recursive(&world_arc, pos, dir, 1);

                *pixel = Color::new(color.x, color.y, color.z, 1.0);
            });

        // Copy pixels to image
        for (i, pixel) in pixels.iter().enumerate() {
            let x = i % width;
            let y = i / width;
            image.set_pixel(x as u32, y as u32, *pixel);
        }


        // Draw image to screen
        let texture = Texture2D::from_image(&image);
        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        next_frame().await;
    }
}





// --- Raycasting ---
fn cast_ray_recursive(
    world: &World,
    pos: Vec3,
    dir: Vec3,
    recursions: u8,
) -> Vec3 {
    
    let new_ray = cast_ray(world, pos, dir);

    let mut color = new_ray.color;



    if recursions == 0 {
        color
    } else {
        let second_color = cast_ray_recursive(world, new_ray.position, new_ray.reflected_dir, recursions - 1);

        let amount_reflected = 0.3;

        let r = second_color.x * color.x;
        let g = second_color.y * color.y;
        let b = second_color.z * color.z;


        color = Vec3::new(r, g, b);

        color
    }
}


fn cast_ray(
    world: &World,
    shoot_pos: Vec3,
    shoot_dir: Vec3,
) -> RayHit {

    let dir: Vec3 = shoot_dir;




    let mut hit = RayHit {
        material: my3d_lib::Material::default(),
        distance: 0.0,
        position: Vec3::zero(),
        direction: shoot_dir,
        reflected_dir: Vec3::new(0.0, 0.0, 0.0),
    };

    let mut shortest_distance = 1000.0;
    let mut hit_tri: Option<&Triangle> = None;

    for mesh in &world.meshes {
        for tri in &mesh.triangles {
            let normal = tri.normals[0];
            let v0 = mesh.position + mesh.vertices[tri.indices[0]];
            let v1 = mesh.position + mesh.vertices[tri.indices[1]];
            let v2 = mesh.position + mesh.vertices[tri.indices[2]];

            let dir_to_normal = dir.dot(normal);
            if dir_to_normal > 0.001 { continue; } // almost parallel to the ray

            let dist = (v0 - shoot_pos).dot(normal) / dir_to_normal;
            if dist < 0.0 { continue; } // plane is behind us

            if dist > shortest_distance { continue; }

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;

            let h = dir.cross(edge2);
            let a = edge1.dot(h);

            if a.abs() < 0.001 { continue; }

            let  f = 1.0 / a;
            let s = shoot_pos - v0;
            let u = f * (s.dot(h));

            if u < 0.0 || u > 1.0 { continue; }

            let q = s.cross(edge1);
            let v = f * (dir.dot(q));

            if v < 0.0 || u + v > 1.0 { continue; }

            shortest_distance = dist;
            hit_tri = Some(tri);
        }
    }

    if !hit_tri.is_some() { return RayHit::default() }

    let hit_tri = hit_tri.unwrap();
    hit.color = hit_tri.color;

    let normal = hit_tri.normals[0];

    let along_normal = normal * shoot_dir.dot(normal);
    let along_tangent = shoot_dir - along_normal;

    hit.reflected_dir = along_tangent - along_normal;

    // add roughness
    if hit_tri.roughness != 0.0 {
        let jx = hash_f32(shortest_distance+0.1) - 0.5;
        let jy = hash_f32(shortest_distance+0.2) - 0.5;
        let jz = hash_f32(shortest_distance+0.3) - 0.5;

        hit.reflected_dir += Vec3::new(jx, jy, jz) * hit_tri.roughness;
    }

    hit.position = shoot_pos + shoot_dir * shortest_distance;


    // Add an artificial sun thing for better lighting
    let sun_dir = Vec3::new(-0.3, 0.5, 0.8).normalized();
    let light = sun_dir.dot(normal)*-0.5 + 0.5;

    hit.color *= light;


    hit
}




// --- Map generation ---
fn generate_map() -> World {
    let mut world = World { meshes: vec![] };

    // Floor
    world.meshes.push(make_plane(10.0));

    world.meshes[0].position = Vec3::new(0.0, 0.0, 1.0);
    world.meshes[0].triangles[0].color = Vec3::new(1.0, 0.0, 0.0);
    world.meshes[0].triangles[1].color = Vec3::new(1.0, 0.0, 0.0);

    // A few cubes at different positions
    let mut cube1 = make_cube(1.0);
    cube1.position = Vec3::new(2.0, 2.0, 0.5);
    world.meshes.push(cube1);

    let mut cube2 = make_cube(0.5);
    cube2.position = Vec3::new(-1.0, 3.0, 0.25);
    for tri in &mut cube2.triangles {
        tri.color = Vec3::new(0.0, 0.0, 1.0);
    }
    world.meshes.push(cube2);

    world

}


pub fn make_cube(size: f32) -> my3d_lib::Mesh {
    let s = size / 2.0;

    // 8 cube corners
    let vertices = vec![
        Vec3::new(-s, -s, -s), // 0
        Vec3::new( s, -s, -s), // 1
        Vec3::new( s,  s, -s), // 2
        Vec3::new(-s,  s, -s), // 3
        Vec3::new(-s, -s,  s), // 4
        Vec3::new( s, -s,  s), // 5
        Vec3::new( s,  s,  s), // 6
        Vec3::new(-s,  s,  s), // 7
    ];

    let color = Vec3::new(1.0, 1.0, 1.0);

    let mut triangles: Vec<Triangle> = Vec::new();

    // helper to add a triangle by vertex indices and normal
    let mut add_tri = |a, b, c, normal: Vec3| {
        triangles.push(Triangle {
            indices: [a, b, c],
            normals: [normal; 3],
            color,
            emission_strength: 0.0,
            emission_color: color,
            roughness: 0.0,
        });
    };

    // Face normals
    let n_front  = Vec3::new(0.0, 0.0, 1.0);
    let n_back   = Vec3::new(0.0, 0.0, -1.0);
    let n_left   = Vec3::new(-1.0, 0.0, 0.0);
    let n_right  = Vec3::new(1.0, 0.0, 0.0);
    let n_top    = Vec3::new(0.0, 1.0, 0.0);
    let n_bottom = Vec3::new(0.0, -1.0, 0.0);

    // Front (+Z)
    add_tri(4, 5, 6, n_front);
    add_tri(4, 6, 7, n_front);

    // Back (-Z)
    add_tri(1, 0, 3, n_back);
    add_tri(1, 3, 2, n_back);

    // Left (-X)
    add_tri(0, 4, 7, n_left);
    add_tri(0, 7, 3, n_left);

    // Right (+X)
    add_tri(5, 1, 2, n_right);
    add_tri(5, 2, 6, n_right);

    // Top (+Y)
    add_tri(3, 7, 6, n_top);
    add_tri(3, 6, 2, n_top);

    // Bottom (-Y)
    add_tri(0, 1, 5, n_bottom);
    add_tri(0, 5, 4, n_bottom);

    my3d_lib::Mesh {
        vertices,
        triangles,
        scale: Vec3::new(1.0, 1.0, 1.0),
        position: Vec3::new(0.0, 0.0, 0.0),
        rotation: Vec3::new(0.0, 0.0, 0.0),
    }
}


fn make_plane(scale: f32) -> my3d_lib::Mesh {
    let s = scale / 2.0;
    let vertices = vec![
        Vec3::new(-s, -s, 0.0),
        Vec3::new(-s, s,  0.0),
        Vec3::new( s, -s, 0.0),
        Vec3::new( s, s,  0.0),
    ];

    let color = Vec3::new(1.0, 1.0, 1.0);
    let mut triangles: Vec<Triangle> = Vec::new();

    let normal = Vec3::new(0.0, 0.0, -1.0);

    triangles.push(Triangle { indices: [0, 1, 2], normals: [normal, normal, normal], color, emission_strength: 0.0, emission_color: color, roughness: 0.0 });
    triangles.push(Triangle { indices: [3, 1, 2], normals: [normal, normal, normal], color, emission_strength: 0.0, emission_color: color, roughness: 0.0 });


    my3d_lib::Mesh { vertices, triangles, scale: Vec3::splat(1.0), position: Vec3::new(0.0, 0.0, 0.0), rotation: Vec3::new(0.0, 0.0, 0.0) }
}



fn hash_f32(x: f32) -> f32 {
    let mut i = x.to_bits();
    i ^= i >> 16;
    i = i.wrapping_mul(0x7feb352d);
    i ^= i >> 15;
    i = i.wrapping_mul(0x846ca68b);
    i ^= i >> 16;

    (i as f32) / (u32::MAX as f32)
}
