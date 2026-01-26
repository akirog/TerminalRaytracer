use std::io::{self, Write};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, cursor, style::{self, Stylize}
};
use rand::random;

const MAP_SIZE: usize = 10;


fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;


    let size = terminal::size()?;

    let map = generate_map();

    for y in 0..size.1 {
        for x in 0..size.0 {

            let map_z: usize = (x / 10) as usize;
            let map_x: usize = (x % 13) as usize;
            let map_y: usize = y as usize;

            if (map_x > MAP_SIZE-1 || map_y > MAP_SIZE-1 || map_z > MAP_SIZE-1) {
                continue
            }

            if map[map_x][map_y][map_z] == 1 {
                stdout
                    .queue(cursor::MoveTo(x,y))?
                    .queue(style::PrintStyledContent( "█".red()))?;
            }


        }
    }
    stdout.flush()?;
    Ok(())
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