use std::fmt::format;
use crate::my3d_lib::*;
use crate::vectors::*;

fn load_file(path: &str) -> (String) {
    std::fs::read_to_string(path).expect("Failed to read file")
}


pub fn parse(path: &str) -> Mesh {
    let mut mesh: Mesh = Mesh::default();
    let text = load_file(path);

    let mut vertex_normals: Vec<Vec3> = vec![];

    for line in text.lines() {
        let parts = line.split(" ").collect::<Vec<&str>>();

        if line.starts_with("v ") {
            // Vertex declaration
            let vert = Vec3::new(
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
                parts[3].parse().unwrap()
            );

            println!("{}", line);
            println!("x: {}, y: {}, z: {}", parts[1], parts[2], parts[3]);

            mesh.vertices.push(vert)
        } else if line.starts_with("vn ") {
            let normal = Vec3::new(
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
                parts[3].parse().unwrap()
            );

            println!("{}", line);
            println!("x: {}, y: {}, z: {}", parts[1], parts[2], parts[3]);


            vertex_normals.push(normal);
        } else if line.starts_with("f ") {
            let mut face: Face = Face::default();

            for (i, part) in parts.iter().enumerate() {
                if i == 0 { continue; } // skip the "f "

                // Faces have three vertices, each with a vertex, texture index and normal index

                part.split("/").enumerate().for_each(|(y, idx)| {
                    if y == 0 {
                        // Vertex index
                        face.indices[i-1] = idx.parse::<usize>().unwrap() - 1;
                    } else if y == 2 {
                        // Vertex normal index
                        face.normals[i-1] = vertex_normals[idx.parse::<usize>().unwrap() - 1];
                    }
                })
            }

            mesh.triangles.push(face);
        }
    }

    // Check if there is a material file, aka a .mtl file to use for material
    let material_path = format!("{}{}", path.rsplit(".obj").last().unwrap(), &*".mtl".to_owned());
    println!("Material path: {:?}", material_path);

    if let text = std::fs::read_to_string(material_path).unwrap().as_str() {
        println!("Material loaded");


    };

    mesh
}