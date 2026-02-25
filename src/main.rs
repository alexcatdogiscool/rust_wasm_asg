
use rand::Rng;
use image::{RgbImage, Rgb};
use std::time::{Instant, Duration};
use rand::seq::SliceRandom;


struct Air_point {
    x: i32,
    y: i32,
    field_strength: f64,
}



struct Bolt<'a> {
    points: Vec<&'a Air_point>,
}

fn dist(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    return (((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f64).powf(0.5);
}

fn basic_dist(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
    return ((x1 - x2).abs() as f64).min((y1 - y2).abs() as f64);
}


fn create_atomosphere(width: i32, height: i32, density: f64) -> Vec<Air_point> {

    let mut atomosphere: Vec<Air_point> = Vec::new();
    let mut rng = rand::thread_rng();

    for y in 0..height {
        for x in 0..width {
            if (density > rng.gen_range(0.0..1.0)) {
                // make an Air_point here
                let point: Air_point = Air_point {
                    x: x,
                    y: y,
                    field_strength: 0.0,
                };
                atomosphere.push(point);
            }
        }
    }

    return atomosphere;
}

fn electrify_atomosphere(atomosphere: &mut Vec<Air_point>, x: u32, y: u32) {
    for point in atomosphere.iter_mut() {
        let dist = dist(x as i32, y as i32, point.x, point.y);
        point.field_strength = 1.0 / (dist.powf(1.0)+1.0);
    }
}


fn get_field_strength(x: i32, y: i32, height: i32) -> f64 {
    // for now assume the ground if lat, so electric field is even acros x.
    // field is 1 at top, 0 at ground.
    return (y as f64) / (height as f64);
}

fn get_relative_field_strength(origin: &Air_point, target: &Air_point) -> f64 {
    // electric feild decays with distance defined by inverse law, (2D, so not inverse square)

    let dist: f64 = dist(origin.x, origin.y, target.x, target.y);
    let decay_ratio: f64 = 1.0 / dist.powf(2.0);
    let field_diff: f64 = origin.field_strength - target.field_strength;
    
    return field_diff * decay_ratio;
}

fn draw_line(img: &mut RgbImage, x0: u32, y0: u32, x1: u32, y1: u32, color: Rgb<u8>) {
    let dx = (x1 as i32 - x0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 as i32 - y0 as i32).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0 as i32;
    let mut y = y0 as i32;

    loop {
        if x >= 0 && y >= 0 && x < img.width() as i32 && y < img.height() as i32 {
            img.put_pixel(x as u32, y as u32, color);
        }
        if x == x1 as i32 && y == y1 as i32 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}


fn main() {

    let mut rng = rand::thread_rng();
    
    let width = 500;
    let height = 500;
    let density: f64 = 0.01;// [density] Air_point's per unit^2. 1 is max, interpreted as probability that a given x,y screen pos constains an Air_point

    let source = (250, 250);

    // create the atomosphere
    let mut atomosphere  = create_atomosphere(width, height, density);

    // give it an electric field
    electrify_atomosphere(&mut atomosphere, source.0, source.1);

    while true {
        let start_time = Instant::now();

        
        

        // init the bolt.
        let mut bolt: Bolt = Bolt {
            points: Vec::new(),
        };
        bolt.points.push(&atomosphere[10]);

        let mut i = 0;
        let mut running: bool = true;
        while running {
            // bolt head starts somewhere
            // get a set of the closest points by relative field strength:
            //      first use basic_dist to narrow down search range
            //      then use propper field strength comparison
            //      chose a random point weighted by relative field strength
            // move bolt head to the selected Air_point
            // repeat.

            let last = bolt.points.last().unwrap();

            let mut sorted_nearby: Vec<&Air_point> = atomosphere.iter()
                .filter(|p| basic_dist(last.x, last.y, p.x, p.y) < 20.0)
                .filter(|p| !bolt.points.iter().any(|bp| std::ptr::eq(*bp, *p)))
                .collect();

            sorted_nearby.sort_by(|a, b| {
                get_relative_field_strength(last, a)
                    .total_cmp(&get_relative_field_strength(last, b))
            });
            // take the best candidate

            bolt.points.push(sorted_nearby.first().expect("empty atomosphere :("));

            i += 1;
            if (dist(bolt.points.last().unwrap().x, bolt.points.last().unwrap().y, source.0 as i32, source.1 as i32) < 10.0) {
                running = false;
            }


        }

        // got the bolt made!
        // now have to draw it to the screen

        let mut img = RgbImage::new(width as u32, height as u32);

        // fill with black background
        for x in 0..width {
            for y in 0..height {
                img.put_pixel(x as u32, y as u32, Rgb([0, 0, 0]));
            }
        }

        // draw the bolt as white lines between consecutive points
        for pair in bolt.points.windows(2) {
            let a = pair[0];
            let b = pair[1];

            draw_line(&mut img, a.x as u32, a.y as u32, b.x as u32, b.y as u32, Rgb([0, 0, 255]));
        }

        for p in atomosphere.iter() {
            img.put_pixel(p.x as u32, p.y as u32, Rgb([255,0,0]));
        }

        img.save("bolt.png").unwrap();
    
        
        println!("fps: {}", 1000.0 / start_time.elapsed().as_millis() as f64);

        return;

    }

}
