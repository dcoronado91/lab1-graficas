use image::{Rgb, RgbImage};

type Point = (i32, i32);

const WIDTH: u32 = 800;
const HEIGHT: u32 = 450;

/// Arista activa para el algoritmo de scanline: rango vertical [y_min, y_max)
/// y la x de intersección en y_min, mas la pendiente inversa dx/dy.
struct Edge {
    y_min: i32,
    y_max: i32,
    x_at_ymin: f64,
    inv_slope: f64,
}

fn build_edges(subpaths: &[Vec<Point>]) -> Vec<Edge> {
    let mut edges = Vec::new();
    for path in subpaths {
        let n = path.len();
        for i in 0..n {
            let (x1, y1) = path[i];
            let (x2, y2) = path[(i + 1) % n];
            if y1 == y2 {
                continue; // las aristas horizontales no aportan intersecciones
            }
            let (top, bottom) = if y1 < y2 {
                ((x1, y1), (x2, y2))
            } else {
                ((x2, y2), (x1, y1))
            };
            let inv_slope = (bottom.0 - top.0) as f64 / (bottom.1 - top.1) as f64;
            edges.push(Edge {
                y_min: top.1,
                y_max: bottom.1,
                x_at_ymin: top.0 as f64,
                inv_slope,
            });
        }
    }
    edges
}

/// Relleno por scanline con regla par-impar (even-odd).
/// Al pasar varios subpaths (ej. un polígono y un agujero) las intersecciones
/// de ambos se combinan y ordenan, así el interior del agujero queda sin pintar.
fn fill_polygons(img: &mut RgbImage, subpaths: &[Vec<Point>], color: Rgb<u8>) {
    let edges = build_edges(subpaths);
    if edges.is_empty() {
        return;
    }
    let y_min = edges.iter().map(|e| e.y_min).min().unwrap();
    let y_max = edges.iter().map(|e| e.y_max).max().unwrap();

    for y in y_min..y_max {
        let mut xs: Vec<f64> = edges
            .iter()
            .filter(|e| y >= e.y_min && y < e.y_max)
            .map(|e| e.x_at_ymin + (y - e.y_min) as f64 * e.inv_slope)
            .collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for pair in xs.chunks_exact(2) {
            let x_start = pair[0].ceil() as i32;
            let x_end = pair[1].ceil() as i32;
            for x in x_start..x_end {
                if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

/// Algoritmo de línea de Bresenham.
fn draw_line(img: &mut RgbImage, p0: Point, p1: Point, color: Rgb<u8>) {
    let (mut x0, mut y0) = p0;
    let (x1, y1) = p1;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && x0 < WIDTH as i32 && y0 >= 0 && y0 < HEIGHT as i32 {
            img.put_pixel(x0 as u32, y0 as u32, color);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn draw_outline(img: &mut RgbImage, path: &[Point], color: Rgb<u8>) {
    let n = path.len();
    for i in 0..n {
        draw_line(img, path[i], path[(i + 1) % n], color);
    }
}

fn main() {
    let mut img = RgbImage::from_pixel(WIDTH, HEIGHT, Rgb([255, 255, 255]));

    let rojo = Rgb([255, 0, 0]);
    let verde = Rgb([0, 128, 0]);
    let naranja = Rgb([255, 140, 0]);
    let amarillo = Rgb([255, 215, 0]);
    let morado = Rgb([128, 0, 128]);
    let negro = Rgb([0, 0, 0]);

    let poligono1: Vec<Point> = vec![
        (165, 380),
        (185, 360),
        (180, 330),
        (207, 345),
        (233, 330),
        (230, 360),
        (250, 380),
        (220, 385),
        (205, 410),
        (193, 383),
    ];

    let poligono2: Vec<Point> = vec![(321, 335), (288, 286), (339, 251), (374, 302)];

    let poligono3: Vec<Point> = vec![(377, 249), (411, 197), (436, 249)];

    let poligono4: Vec<Point> = vec![
        (413, 177),
        (448, 159),
        (502, 88),
        (553, 53),
        (535, 36),
        (676, 37),
        (660, 52),
        (750, 145),
        (761, 179),
        (672, 192),
        (659, 214),
        (615, 214),
        (632, 230),
        (580, 230),
        (597, 215),
        (552, 214),
        (517, 144),
        (466, 180),
    ];

    // Agujero dentro del polígono 4: no debe pintarse.
    let poligono5_agujero: Vec<Point> = vec![(682, 175), (708, 120), (735, 148), (739, 170)];

    fill_polygons(&mut img, &[poligono1.clone()], rojo);
    fill_polygons(&mut img, &[poligono2.clone()], verde);
    fill_polygons(&mut img, &[poligono3.clone()], naranja);
    fill_polygons(
        &mut img,
        &[poligono4.clone(), poligono5_agujero.clone()],
        amarillo,
    );

    draw_outline(&mut img, &poligono1, negro);
    draw_outline(&mut img, &poligono2, negro);
    draw_outline(&mut img, &poligono3, negro);
    draw_outline(&mut img, &poligono4, negro);
    draw_outline(&mut img, &poligono5_agujero, morado);

    img.save("out.png").expect("No se pudo guardar out.png");
    println!("out.png generado correctamente ({}x{})", WIDTH, HEIGHT);
}
