extern crate core;


use macroquad::prelude::*;

use std::sync::{Arc};
use std::sync::atomic::{AtomicU32, Ordering};
use rayon::prelude::*;

pub mod vectors;
mod my3d_lib;
mod obj_parser;

use crate::vectors::Vec3;
use crate::my3d_lib::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Macro quad Raytracer".to_owned(),
        window_width: 600,
        window_height: 400,
        ..Default::default()
    }
}


static FRAME: AtomicU32 = AtomicU32::new(0);


#[macroquad::main(window_conf)]
async fn main() {
    // Create map
    let world: World  = generate_map();
    let world_arc = Arc::new(world); // Move map into Arc

    // Camera state
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let mut yaw: f32 = 0.0;
    let mut pitch: f32 = 0.0;

    let mut focal_distance: f32 = 4.0;
    let mut aperture_radius = 0.05;

    let speed: f32 = 2.0;
    let mouse_sensitivity: f32 = 0.2;

    // Hide cursor and capture
    set_cursor_grab(true);
    show_mouse(false);


    let width = screen_width() as usize;
    let height = screen_height() as usize;
    let width_ratio = width as f32 / height as f32;

    let mut accumulation_buffer = vec![Vec3::default(); width * height];
    let mut sample_count = 0u32;

    let mut debug_info: bool = true;
    let mut input_locked: bool = false;
    let mut display_image: bool = true;
    let mut change_focal: bool = false;


    let mut forward: Vec3 = Vec3::default();
    forward.x = f32::sin(yaw) * f32::cos(pitch);
    forward.y = f32::cos(yaw) * f32::cos(pitch);
    forward.z = f32::sin(pitch);

    let world_up = Vec3::new(0.0, 0.0, 1.0);

    let mut right = forward.cross(world_up).normalize();
    let mut up = right.cross(forward).normalize();


    loop {
        clear_background(BLACK);

        // Update frame
        let frame = FRAME.fetch_add(1, Ordering::Relaxed);


        if is_key_pressed(KeyCode::G) { debug_info ^= true; };
        if is_key_pressed(KeyCode::L) { input_locked ^= true; };
        if is_key_pressed(KeyCode::U) { display_image ^= true; };
        if is_key_pressed(KeyCode::R) { change_focal ^= true; };

        if is_key_down(KeyCode::Escape) { break; }

        // --- Input ---
        if !input_locked {
            let delta_pos = mouse_delta_position();
            yaw -= delta_pos.x * mouse_sensitivity;
            pitch -= delta_pos.y * mouse_sensitivity;



            forward.x = f32::sin(yaw) * f32::cos(pitch);
            forward.y = f32::cos(yaw) * f32::cos(pitch);
            forward.z = f32::sin(pitch);

            right = forward.cross(world_up).normalize();
            up = right.cross(forward).normalize();



            let dt = get_frame_time();
            let amount = speed * dt;

            if is_key_down(KeyCode::W) { pos += forward * amount; }
            if is_key_down(KeyCode::S) { pos -= forward * amount; }

            if is_key_down(KeyCode::D) { pos += right * amount; }
            if is_key_down(KeyCode::A) { pos -= right * amount; }

            if is_key_down(KeyCode::Space)     { pos -= up * amount; }
            if is_key_down(KeyCode::LeftShift) { pos += up * amount; }


            if mouse_wheel().1 > 0.0 {
                if change_focal {
                    focal_distance += mouse_wheel().1/1000.0;
                } else {
                    aperture_radius += mouse_wheel().1/10000.0;
                }
            }



            if get_keys_down().len() > 0 || mouse_delta_position() != Vec2::default() || mouse_wheel().1 != 0.0 {
                // Halve accumulation
                accumulation_buffer = vec![Vec3::default(); width * height];
                sample_count = 0;
            }
        }



        // --- Rendering ---
        let mut image = Image::gen_image_color(width as u16, height as u16, BLACK);

        // Create pixel buffer
        let mut pixels = vec![Color::default(); width * height];


        let inv_width = 1.0 / width as f32;
        let inv_height = 1.0 / height as f32;
        let focal_pos = pos + forward * focal_distance;

        // Parallel iteration over pixels
        pixels.par_iter_mut()
            .enumerate()
            .for_each(|(index, pixel)| {
                let x = index % width;
                let y = index / width;

                // Get from -0.5 to 0.5
                let screen_x = (x as f32 * inv_width - 0.5) * 2.0 * width_ratio;
                let screen_y = (y as f32 * inv_height - 0.5) * 2.0;


                let image_plane_center = pos + forward * focal_distance;

                let pixel_world =
                    image_plane_center
                        + screen_x * right
                        + screen_y * up;

                // focus target
                let focal_point = pixel_world;

                // random lens offset
                let mut seed = hash_u32(frame);
                seed ^= screen_x as u32;
                seed ^= screen_y as u32;

                let disk = random_in_unit_disk(seed);

                let lens_offset =
                    right * disk.x * aperture_radius +
                        up    * disk.y * aperture_radius;

                let ray_origin = pos + lens_offset;
                let ray_dir = (focal_point - ray_origin).normalize();



                let recursions: u8 = 4;

                let color = cast_ray_recursive(&world_arc, ray_origin, ray_dir, recursions, frame, x, y);

                let color = color.unwrap_or(Vec3::default());

                *pixel = Color::new(color.x, color.y, color.z, 1.0);
            });

        sample_count += 1;

        // Copy pixels to image
        for (i, pixel) in pixels.iter().enumerate() {
            // Add to accumulation buffer
            accumulation_buffer[i].x += pixel.r;
            accumulation_buffer[i].y += pixel.g;
            accumulation_buffer[i].z += pixel.b;

            if display_image {
                // Compute average
                let avg = accumulation_buffer[i] / (sample_count as f32);


                let x = i % width;
                let y = i / width;
                image.set_pixel(x as u32, y as u32,
                                Color::new(avg.x, avg.y, avg.z, 1.0)
                );
            }
        }




        if display_image {
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
        }

        // if debug is on draw some debug stuff
        if debug_info {
            draw_text(&*format!("Sample count: {}", sample_count), 5.0, 20.0, 20.0, WHITE);
            draw_text(&*format!("FPS: {}", get_fps()), 5.0, 40.0, 20.0, WHITE);
            draw_text(&*format!("Focal distance: {}", focal_distance), 5.0, 60.0, 20.0, WHITE);
            draw_text(&*format!("Aperture radius: {}", aperture_radius), 5.0, 80.0, 20.0, WHITE);

            let ideal_distance = cast_ray(&world_arc, pos, forward, frame, 0, 0, 0).unwrap_or(RayHit::default()).distance;
            draw_text(&*format!("Ideal focal distance: {}", ideal_distance), 5.0, 100.0, 20.0, WHITE);
        }

        next_frame().await;
    }
}




// --- Raycasting ---
fn cast_ray_recursive(
    world: &World,
    pos: Vec3,
    dir: Vec3,
    recursions: u8,
    frame: u32,
    screen_x: usize,
    screen_y: usize,
) -> Option<Vec3> {
    
    let new_ray = cast_ray(world, pos, dir, frame, screen_x, screen_y, recursions);

    if new_ray.is_none() { return None }
    let new_ray = new_ray.unwrap();

    if recursions == 0 {
        // If this is the last thing we hit we only return the light it emits since no further light could hit it
        Some(new_ray.material.emission)

    } else {
        // Here we get the color of the light ray when it hits us
        let received_color = cast_ray_recursive(world, new_ray.position, new_ray.reflected_dir, recursions - 1, frame, screen_x, screen_y);

        if received_color.is_none() { return Some( new_ray.material.emission ); }
        let received_color = received_color.unwrap();

        // The amount of light this ray now has is the light we just got times our albedo, aka how much we reflect
        let mut color = received_color * new_ray.material.albedo;

        let emitted_light = new_ray.material.emission;

        color = color + emitted_light;


        Some(color)
    }
}


fn cast_ray(
    world: &World,
    shoot_pos: Vec3,
    shoot_dir: Vec3,
    frame: u32,
    screen_x: usize,
    screen_y: usize,
    recursions: u8,
) -> Option<RayHit> {

    let dir: Vec3 = shoot_dir;


    let mut hit = RayHit {
        material: my3d_lib::Material::default(),
        distance: 0.0,
        position: Vec3::default(),
        direction: shoot_dir,
        reflected_dir: Vec3::new(0.0, 0.0, 0.0),
    };

    let mut shortest_distance = 1000.0;
    let mut hit_tri: Option<&Face> = None;

    for mesh in &world.baked_meshes {
        for tri in &mesh.faces {
            let normal = tri.normals[0];
            let v0 = mesh.vertices[tri.indices[0]];

            let dir_to_normal = dir.dot(normal);
            if dir_to_normal > 0.001 { continue; } // almost parallel to the ray

            let dist = (v0 - shoot_pos).dot(normal) / dir_to_normal;
            if dist < 0.0 { continue; } // plane is behind us

            if dist > shortest_distance { continue; }

            let edge1 = tri.edges[0];
            let edge2 = tri.edges[1];

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
            hit.material = mesh.materials.get(tri.material_idx).unwrap().clone();
        }
    }

    if !hit_tri.is_some() { return None }


    let hit_tri = hit_tri.unwrap();
    let normal = hit_tri.normals[0];

    let along_normal = normal * shoot_dir.dot(normal);
    let along_tangent = shoot_dir - along_normal;

    hit.reflected_dir = along_tangent - along_normal;

    // add roughness
    if hit.material.roughness != 0.0 {
        let roughness = hit.material.roughness * hit.material.roughness;


        let mut seed = hash_u32(frame);
        seed = hash_u32(seed + screen_x as u32);
        seed = hash_u32(seed + screen_y as u32);
        seed = hash_u32(seed + recursions as u32);

        let mut random_dir = random_vec3(seed);

        if random_dir.dot(normal) < 0.0 {
            random_dir = random_dir * -1.0;
        }

        hit.reflected_dir = hit.reflected_dir + (random_dir - hit.reflected_dir) * roughness;
        hit.reflected_dir = hit.reflected_dir.normalize();
    }

    hit.position = shoot_pos + shoot_dir * shortest_distance;

    Some(hit)
}




// --- Map generation ---
fn generate_map() -> World {
    let mut world = World { meshes: vec![], baked_meshes: vec![] };

    // // Floor
    // world.meshes.push(make_plane(10.0));
    //
    // world.meshes[0].position = Vec3::new(0.0, 0.0, 1.0);
    // world.meshes[0].material.albedo = Vec3::new(0.22, 0.5, 0.02);
    // world.meshes[0].material.roughness = 1.0;
    //
    // // A few cubes
    // let mut cube1 = make_cube(1.0);
    // cube1.position = Vec3::new(2.0, 2.0, 0.5);
    // cube1.material.albedo = Vec3::new(1.0, 0.0, 0.0);
    // cube1.material.roughness = 0.2;
    // world.meshes.push(cube1);
    //
    // let mut cube2 = make_cube(0.5);
    // cube2.position = Vec3::new(-1.0, 2.0, -0.2);
    // cube2.material.emission_color = Vec3::new(1.0, 1.0, 1.0);
    // cube2.material.emission_strength = 10.0;
    //
    // world.meshes.push(cube2);
    //
    // let mut cube3 = make_cube(1.0);
    // cube3.position = Vec3::new(-1.0, -2.0, 0.5);
    // cube3.material.albedo = Vec3::new(0.0, 0.0, 0.0);
    // cube3.material.emission_color = Vec3::new(0.0, 1.0, 0.0);
    // cube3.material.emission_strength = 3.0;
    //
    // world.meshes.push(cube3);
    //
    //
    // let mut cube3 = make_cube(1.0);
    // cube3.position = Vec3::new(-1.0, -4.0, 0.5);
    // cube3.material.albedo = Vec3::new(0.0, 0.0, 0.0);
    // cube3.material.emission_color = Vec3::new(1.0, 0.0, 0.0);
    // cube3.material.emission_strength = 3.0;
    //
    // world.meshes.push(cube3);
    //
    //
    // let mut cube4 = make_cube(1.0);
    // cube4.position = Vec3::new(2.0, 3.2, 0.5);
    // cube4.material.albedo = Vec3::new(0.2, 1.0, 0.1);
    // cube4.material.roughness = 0.1;
    // cube4.material.emission_color = Vec3::new(0.4, 1.0, 0.4);
    // cube4.material.emission_strength = 1.0;
    // world.meshes.push(cube4);


    // Add a .obj file
    world.meshes.extend( obj_parser::load_glb("src/models/cornell_box.glb") );


    world.bake_meshes();
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

    let mut faces: Vec<Face> = Vec::new();

    // helper to add a triangle by vertex indices and normal
    let mut add_tri = |a, b, c, normal: Vec3| {
        faces.push(Face {
            indices: [a, b, c],
            normals: [normal; 3],
            material_idx: 0,
            ..Default::default()
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
        faces,
        scale: Vec3::new(1.0, 1.0, 1.0),
        position: Vec3::new(0.0, 0.0, 0.0),
        rotation: ::glam::Quat::default(),
        materials: vec![my3d_lib::Material::default()],
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

    let mut faces: Vec<Face> = Vec::new();

    let normal = Vec3::new(0.0, 0.0, -1.0);

    faces.push(Face { indices: [0, 1, 2], normals: [normal, normal, normal], material_idx: 0, ..Default::default() });
    faces.push(Face { indices: [3, 1, 2], normals: [normal, normal, normal], material_idx: 0, ..Default::default() });


    my3d_lib::Mesh {
        vertices,
        faces,
        scale: Vec3::splat(1.0),
        position: Vec3::new(0.0, 0.0, 0.0),
        rotation: ::glam::Quat::default(),
        materials: vec![my3d_lib::Material::default()],
    }
}



#[inline(always)]
fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    x
}

#[inline(always)]
fn hash_f32(x: u32) -> f32 {
    (hash_u32(x) as f32) * (1.0 / u32::MAX as f32)
}


fn random_vec3(seed: u32) -> Vec3 {
    let x = hash_f32(seed ^ 0xA2C79B3D) * 2.0 - 1.0;
    let y = hash_f32(seed ^ 0xC3E5F7A1) * 2.0 - 1.0;
    let z = hash_f32(seed ^ 0xB492F11D) * 2.0 - 1.0;

    Vec3::new(x, y, z)
}


fn random_in_unit_disk(seed: u32) -> Vec3 {
    let r1 = hash_f32(seed);
    let r2 = hash_f32(seed ^ 0x9E3779B9);

    let r = r1.sqrt();
    let theta = r2 * std::f32::consts::TAU;

    Vec3::new(r * theta.cos(), r * theta.sin(), 0.0)
}
