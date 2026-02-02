extern crate core;

use macroquad::prelude::*;

use std::thread;
use std::sync::{Arc, Mutex};


pub mod vectors;
mod my3d_lib;

use crate::vectors::Vec3;
use crate::my3d_lib::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Macroquad Raytracer".to_owned(),
        window_width: 500,
        window_height: 500,
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

        // Wrap mutable data in Arc<Mutex<>> for thread safety
        let pixels = Arc::new(Mutex::new(vec![Color::default(); width * height]));

        let num_threads = 4;
        let rows_per_thread = (height + num_threads - 1) / num_threads;

        let mut handles = vec![];


        for thread_id in 0..num_threads {
            let pixels = Arc::clone(&pixels);
            let world = Arc::clone(&world_arc);


            let pos = pos;

            let start_row = thread_id * rows_per_thread;
            let end_row = (start_row + rows_per_thread).min(height);

            let handle = thread::spawn(move || {
                for y in start_row..end_row {
                    for x in 0..width {
                        // Get from -0.5 to 0.5
                        let screen_x = ((x as f32 / width as f32) - 0.5) * 2.0;
                        let screen_y = ((y as f32 / height as f32) - 0.5) * 2.0;

                        let mut dir = forward.clone();

                        dir += right * screen_x * 1.0;
                        dir += up * screen_y * 1.0;

                        dir = dir.normalized();

                        let color = cast_ray_recursive(&world, pos, dir, 1);

                        let mut pixels = pixels.lock().unwrap();
                        pixels[y * width + x] = Color::new(color.x, color.y, color.z, 1.0);
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }

        // Copy pixels back to image
        let pixels = pixels.lock().unwrap();
        for y in 0..height {
            for x in 0..width {
                image.set_pixel(x as u32, y as u32, pixels[y * width + x]);
            }
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
        let second_color = cast_ray_recursive(world, new_ray.position + (pos-new_ray.position) * 0.001, new_ray.reflected_dir, recursions - 1);

        let amount_reflected = 0.1;

        let r = (color.x * (1.0-amount_reflected) + second_color.x * amount_reflected);
        let g = (color.y * (1.0-amount_reflected) + second_color.y * amount_reflected);
        let b = (color.z * (1.0-amount_reflected) + second_color.z * amount_reflected);


        color = Vec3::new(r, g, b);

        color
    }
}


fn cast_ray(
    world: &World,
    shoot_pos: Vec3,
    shoot_dir: Vec3,
) -> RayHit {


    let mut pos: Vec3 = shoot_pos;
    let steps = 6;

    let mut hit = RayHit {
        color: Vec3::new(1.0, 1.0, 1.0),
        position: shoot_pos,
        direction: shoot_dir,
        reflected_dir: Vec3::new(0.0, 0.0, 0.0),
    };

    hit
}




// --- Map generation ---
fn generate_map() -> World {
    let mut world = World { meshes: vec![] };

    // create a cube
    world.meshes.push(make_cube(1.0));

    world
}



pub struct Triangle {
    pub indices: [usize; 3],
    pub normals: [Vec3; 3],
    pub color: Vec3,
}

pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<Triangle>,
    pub scale: Vec3,
    pub position: Vec3,
    pub rotation: Vec3,
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

    let mut triangles: Vec<my3d_lib::Triangle> = Vec::new();

    // helper to add a triangle by vertex indices and normal
    let mut add_tri = |a, b, c, normal: Vec3| {
        triangles.push(my3d_lib::Triangle {
            indices: [a, b, c],
            normals: [normal; 3],
            color,
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
