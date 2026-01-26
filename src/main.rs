extern crate core;

use std::cmp::{max, min};
use std::mem::forget;
use std::ops::SubAssign;
use macroquad::prelude::*;

use std::thread;
use std::sync::{Arc, Mutex};


mod vectors;
use crate::vectors::Vec3;

const MAP_SIZE: usize = 16;

fn window_conf() -> Conf {
    Conf {
        window_title: "Macroquad Raymarcher".to_owned(),
        window_width: 500,
        window_height: 500,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Create map
    let map = generate_map();
    let map_arc = Arc::new(map); // Move map into Arc

    // Player state
    let mut pos = Vec3::new(8.5, 8.5, 8.5);
    let speed: f32 = 2.0;
    let mut yaw: f32 = 0.0;
    let mut pitch: f32 = 0.0;
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

        let num_threads = 4; // Adjust based on your CPU cores
        let rows_per_thread = (height + num_threads - 1) / num_threads;

        let mut handles = vec![];


        for thread_id in 0..num_threads {
            let pixels = Arc::clone(&pixels);
            let map = Arc::clone(&map_arc);


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

                        let color = cast_ray(&map, pos, dir);

                        let mut pixels = pixels.lock().unwrap();
                        pixels[y * width + x] = color;
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
fn cast_ray(
    map: &[[[u8; MAP_SIZE]; MAP_SIZE]; MAP_SIZE],
    mut pos: Vec3,
    dir: Vec3,
) -> Color {


    let step_size = 0.08;
    let step = dir * step_size;

    let mut dist: f32 = 0.0;

    let steps = 50;
    // First check x borders
    for _ in 0..steps {
        pos += step;

        dist += step_size;

        // if pos.x < 0.0 || pos.x >= MAP_SIZE as f32 ||
        //     pos.y < 0.0 || pos.y >= MAP_SIZE as f32 ||
        //     pos.z < 0.0 || pos.z >= MAP_SIZE as f32 {
        //     continue;
        // }

        let xi = pos.x as usize % MAP_SIZE;
        let yi = pos.y as usize % MAP_SIZE;
        let zi = pos.z as usize % MAP_SIZE;

        if map[xi][yi][zi] > 0 {
            // Map position to color
            let grid_amount = 0.25;

            let max_dist = steps as f32 * step_size;

            let dist = (1.0-(dist/max_dist)).sqrt();

            let dist = if dist < 0.0 {0.0} else {dist};
            let r = ((pos.x % grid_amount) * 255.0 * (1.0 / grid_amount) * dist) as u8;
            let g = ((pos.y % grid_amount) * 255.0 * (1.0 / grid_amount) * dist) as u8;
            let b = ((pos.z % grid_amount) * 255.0 * (1.0 / grid_amount) * dist) as u8;
            return Color::from_rgba(r, g, b, 255);
        }
    }

    BLACK
}

// --- Map generation ---
fn generate_map() -> [[[u8; MAP_SIZE]; MAP_SIZE]; MAP_SIZE] {
    let mut map = [[[0; MAP_SIZE]; MAP_SIZE]; MAP_SIZE];

    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            for z in 0..MAP_SIZE {
                if x != 16 || z != 16 {
                    let val = ::rand::random::<u8>();
                    map[x][y][z] = if val < 200 {0} else {1};
                } else {
                    map[x][y][z] = 0;
                }
            }
        }
    }

    map
}
