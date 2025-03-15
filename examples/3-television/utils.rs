use std::{
    cmp::Ordering,
    fs::read,
    io::{BufRead, Cursor},
    path::Path,
};

// Read a PNG file into a (width, height) tuple and a byte vector
pub fn read_png_data(path: &str) -> ((usize, usize), Vec<u8>) {
    let texture_data = image::open(path).unwrap().into_rgb8();
    let pixels: Vec<u8> = texture_data.pixels().flat_map(|p| p.0).collect();
    (
        (
            texture_data.width() as usize,
            texture_data.height() as usize,
        ),
        pixels,
    )
}
// Read a wavefront OBJ file to a data vector
// Each vertex is formatted as [X, Y, Z, NX, NY, NZ, U, V]
pub fn read_obj(path: &Path) -> Vec<[f32; 8]> {
    let obj_contents = read(path).unwrap();
    let mut vertices = Vec::<[f32; 3]>::new();
    let mut texture_coords = Vec::<[f32; 2]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut faces = Vec::<[[usize; 3]; 3]>::new();

    let mut lines = Cursor::new(obj_contents).lines();
    while let Some(Ok(line)) = lines.next() {
        let mut tokens = line.trim().split(' ');
        // Utility macro to get the next token as a string and parse it to whatever the compiler expects
        macro_rules! next {
            () => {
                tokens.next().unwrap().parse().unwrap()
            };
        }
        let char = tokens.next().unwrap();
        if char.starts_with('#') || char.is_empty() {
            continue;
        }
        match char {
            "v" => vertices.push([next!(), next!(), next!()]),
            "vt" => texture_coords.push([next!(), next!()]),
            "vn" => normals.push([next!(), next!(), next!()]),
            "f" => faces.push(core::array::from_fn(|_| {
                let mut val = tokens.next().unwrap().split("/");
                core::array::from_fn(|_| val.next().unwrap().parse().unwrap())
            })),
            // Ignored
            "g" => {}
            "mtllib" => {}
            "usemtl" => {}
            _ => println!("Unknown {}", line),
        }
    }
    macro_rules! get {
        ($data: ident, $func: ident) => {
            core::array::from_fn(|i| {
                $data
                    .iter()
                    .$func(|a, b| {
                        if a[i] > b[i] {
                            Ordering::Greater
                        } else {
                            Ordering::Less
                        }
                    })
                    .unwrap()[i]
            })
        };
    }
    let max_vert: [f32; 3] = get!(vertices, max_by);
    let min_vert: [f32; 3] = get!(vertices, min_by);
    let max_tex: [f32; 2] = get!(texture_coords, max_by);
    let min_tex: [f32; 2] = get!(texture_coords, min_by);
    println!(
        "Max vertice: {:?}, min vertice: {:?}, max texture coord: {:?}, min texture coord: {:?}",
        max_vert, min_vert, max_tex, min_tex
    );
    let divisor = 500.0;
    // Build an array of [X, Y, Z, U, V] for each vertice
    let mut data = Vec::<[f32; 8]>::with_capacity(faces.len());
    // Calculate offets to center mesh around (0, 0)
    let offsets: [f32; 3] = [-97.96449, 530.126, -37.828995];
    // core::array::from_fn(|i| (max_vert[i] + min_vert[i]) / 2.0);
    println!("Offsets: {:?}", offsets);
    faces.into_iter().for_each(|face| {
        face.into_iter().for_each(|f| {
            let v = vertices[f[0] - 1];
            let t = texture_coords[f[1] - 1];
            let n = normals[f[2] - 1];
            data.push([
                (v[0] - offsets[0]) / divisor,
                (v[1] - offsets[1]) / divisor,
                (v[2] - offsets[2]) / divisor,
                n[0],
                n[1],
                n[2],
                t[0],
                t[1],
            ]);
        })
    });
    data
}
