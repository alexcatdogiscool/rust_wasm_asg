
use rand::Rng;


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

    for x in 0..width {
        for y in 0..height {
            if (density > rng.gen_range(0.0..1.0)) {
                // make an Air_point here
                let point: Air_point = Air_point {
                    x: x,
                    y: y,
                    field_strength: get_field_strength(x, y, height),
                };
                atomosphere.push(point);
            }
        }
    }

    return atomosphere;
}


fn get_field_strength(x: i32, y: i32, height: i32) -> f64 {
    // for now assume the ground if lat, so electric field is even acros x.
    // field is 1 at top, 0 at ground.
    return (y as f64) / (height as f64);
}

fn get_relative_field_strength(origin: &Air_point, target: &Air_point) -> f64 {
    // electric feild decays with distance defined by inverse law, (2D, so not inverse square)

    let dist: f64 = dist(origin.x, origin.y, target.x, target.y);
    let decay_ratio: f64 = 1.0 / dist;
    let field_diff: f64 = origin.field_strength - target.field_strength;
    
    return field_diff * decay_ratio;
}


fn main() {
    
    let width = 100;
    let height = 100;
    let density: f64 = 0.1;// [density] Air_point's per unit^2. 1 is max, interpreted as probability that a given x,y screen pos constains an Air_point

    // create the atomosphere
    let mut atomosphere  = create_atomosphere(width, height, density);

    // init the bolt.
    let mut bolt: Bolt = Bolt {
        points: Vec::new(),
    };
    bolt.points.push(&atomosphere[0]);

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

        let mut sorted_atompshphere: Vec<&Air_point> = {// sorth the atomosphere points by electric field
            let last = bolt.points.last().unwrap();
            let mut refs: Vec<&Air_point> = atomosphere.iter().collect();

            refs.sort_by(|a, b| {
                get_relative_field_strength(last, a).total_cmp(&get_relative_field_strength(last, b))
            });

            refs
        };
        sorted_atompshphere.retain(|p| !bolt.points.iter().any(|bp| std::ptr::eq(*bp, *p)));// remove all points that bolt has already touched
        
        // take the best candidate
        bolt.points.push(sorted_atompshphere.first().expect("empty atomosphere :("));

        i += 1;
        if (i == 100) {
            running = false;
        }


    }

    // got the bolt made!
    // now have to draw it to the screen

}
