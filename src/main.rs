mod vectors;

use std::io::{self, Write};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, cursor, style::{self, Stylize}
};
use crossterm::style::Color;
use rand::random;

use crate::vectors::Vec3;

const MAP_SIZE: usize = 32;


fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;


    let size = terminal::size()?;

    let map = generate_map();

    for y in 0..size.1 {
        for x in 0..size.0 {

            // Make them 0 to 1
            let mut yaw: f32 = x as f32 / size.0 as f32;
            let mut pitch: f32 = y as f32 / size.1 as f32;

            // Make them -0.5 to 0.5
            yaw -= 0.5;
            pitch -= 0.5;

            pitch *= size.1 as f32 / size.0 as f32;

            yaw = yaw * 2.0;
            pitch = pitch * 2.0;

            let color: Color = cast_ray(&map, yaw, pitch);

            stdout
                .queue(cursor::MoveTo(x,y))?
                .queue(style::PrintStyledContent( "█".with(color)))?;



        }
    }
    stdout.flush()?;
    Ok(())
}




fn cast_ray(map: &[[[i32; MAP_SIZE]; MAP_SIZE]; MAP_SIZE], yaw: f32, pitch: f32) -> Color {
    let mut color = Color::Rgb {r: 0, g: 0, b: 0};

    let mut pos: Vec3 = Vec3::new(16.0, -4.0, 16.0);
    let mut dir: Vec3 = Vec3::zero();

    let mut brightness: f32 = 1.0;

    // set up dir vector
    dir.x = f32::sin(yaw) * f32::cos(pitch);
    dir.y = f32::cos(yaw) * f32::cos(pitch);
    dir.z = f32::sin(pitch);

    let step_size: f32 = 0.1;
    let step: Vec3 = dir*step_size;

    for i in 0..100 {
        // Increment pos
        pos += step;

        // Decrement brightness
        brightness -= step_size/10.0;

        // Continue if we are outside the bounds
        if pos.x < 0.0 || pos.x >= MAP_SIZE as f32 ||
            pos.y < 0.0 || pos.y >= MAP_SIZE as f32 ||
            pos.z < 0.0 || pos.z >= MAP_SIZE as f32 {

            continue;
        }

        let xi = pos.x as usize;
        let yi = pos.y as usize;
        let zi = pos.z as usize;


        // If we're in a block stop checking
        if map[xi][yi][zi] > 0 {
            // Break out of the loop
            color = Color::Rgb {r: (brightness * 255.0) as u8, g: (brightness * 255.0) as u8, b: (brightness * 255.0) as u8};
            break;
        }
    }

    color
}



fn generate_map() -> [[[i32; MAP_SIZE]; MAP_SIZE]; MAP_SIZE] {
    let mut map: [[[i32; MAP_SIZE]; MAP_SIZE]; MAP_SIZE] = [[[0; MAP_SIZE]; MAP_SIZE]; MAP_SIZE];


    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            for z in 0..MAP_SIZE {
                map[x][y][z] = random::<i32>()%2;

            }
        }
    }

    map
}