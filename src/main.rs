use std::{time::Instant};

#[derive(Debug)]
enum Quadtree<T: Clone> {
    Empty,
    Value(T),
    // NW NE SW SE
    Subdivision(Box<[Quadtree<T>; 4]>),
}

#[derive(Debug)]
struct SpatialData<T: Clone> {
    data: T,
    position: (f32, f32),
}

impl<T: Clone> Quadtree<T> {
    fn insert(&mut self, value: SpatialData<T>) {
        let pos = (0.5, 0.5);
        self._insert(pos, value);
    }

    fn _insert(&mut self, pos: (f32, f32), value: SpatialData<T>) {
        match self {
            Quadtree::Empty => *self = Quadtree::Value(value.data),
            Quadtree::Subdivision(sd) => {
                let vpos = value.position;
                match ((vpos.0 < pos.0), (vpos.1 < pos.1)) {
                    (false, false) => sd[0]._insert((pos.0 * 1.5, pos.1 * 1.5), value),
                    (false, true) => sd[1]._insert((pos.0 * 1.5, pos.1 / 2.0), value),
                    (true, false) => sd[2]._insert((pos.0 / 2.0, pos.1 * 1.5), value),
                    (true, true) => sd[3]._insert((pos.0 / 2.0, pos.1 / 2.0), value),
                };
            }
            Quadtree::Value(data) => {
                let mut sd = Quadtree::Subdivision(Box::new([
                    Quadtree::Empty,
                    Quadtree::Empty,
                    Quadtree::Empty,
                    Quadtree::Empty,
                ]));
                sd._insert(
                    pos,
                    SpatialData {
                        data: data.clone(),
                        position: pos,
                    },
                );
                sd._insert(pos, value);
                *self = sd;
            }
        }
    }
}

fn main() {
    // dbg!(size_of::<Box<[Quadtree<()>; 4]>>());

    // let qt: Quadtree<_> = Quadtree::Value(0x69696969);
    // let qt: Quadtree<f32> = Quadtree::Subdivision(Box::new([Quadtree::Empty, Quadtree::Empty, Quadtree::Empty, Quadtree::Empty]));
    let qt: Quadtree<f32> = Quadtree::Empty;

    let ptr: *const u128 = unsafe { std::mem::transmute(&qt) };
    println!("{:x}", unsafe { ptr.read() });

    return;

    let st = Instant::now();

    const N: usize = 100;

    let values = (0..N)
        .flat_map(|x| {
            (0..N).map(move |y| SpatialData {
                position: (x as f32 / 100.0, y as f32 / 100.0),
                data: x * y,
            })
        })
        .collect::<Vec<SpatialData<usize>>>();

    let mut qt = Quadtree::Empty;

    for value in values {
        qt.insert(value);
    }

    let et = Instant::now();

    println!("Time: {:?}", et.duration_since(st));
}
