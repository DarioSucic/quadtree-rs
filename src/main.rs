#![feature(allocator_api)]

use std::alloc::Allocator;
use std::{mem::size_of, time::Instant};

use bumpalo::Bump;

struct Quadtree<'a, T: Clone, A>
where
    &'a A: Allocator,
{
    root: QTInner<'a, T, A>,
    storage: &'a A,
}

#[derive(Debug)]
enum QTInner<'a, T: Clone, A>
where
    &'a A: Allocator,
{
    Empty,
    Value(T),
    // NW NE SW SE
    Subdivision(Box<[QTInner<'a, T, A>; 4], &'a A>),
}

#[derive(Debug)]
struct SpatialData<T: Clone> {
    data: T,
    position: (f32, f32),
}

impl<'a, T: Clone, A> Quadtree<'a, T, A>
where
    &'a A: Allocator,
{
    fn new_in(alloc: &'a A) -> Self {
        Self {
            root: QTInner::Empty,
            storage: alloc,
        }
    }

    fn insert(&mut self, value: SpatialData<T>) {
        let pos = (0.5, 0.5);
        self.root._insert(pos, value, &self.storage);
    }
}

impl<'a, T: Clone, A> QTInner<'a, T, A>
where
    &'a A: Allocator,
{
    fn _insert(&mut self, pos: (f32, f32), value: SpatialData<T>, alloc: &'a A) {
        match self {
            QTInner::Empty => *self = QTInner::Value(value.data),
            QTInner::Subdivision(sd) => {
                let vpos = value.position;
                match ((vpos.0 < pos.0), (vpos.1 < pos.1)) {
                    (false, false) => sd[0]._insert((pos.0 * 1.5, pos.1 * 1.5), value, alloc),
                    (false, true) => sd[1]._insert((pos.0 * 1.5, pos.1 / 2.0), value, alloc),
                    (true, false) => sd[2]._insert((pos.0 / 2.0, pos.1 * 1.5), value, alloc),
                    (true, true) => sd[3]._insert((pos.0 / 2.0, pos.1 / 2.0), value, alloc),
                };
            }
            QTInner::Value(data) => {
                let mut sd = QTInner::Subdivision(Box::new_in(
                    [
                        QTInner::Empty,
                        QTInner::Empty,
                        QTInner::Empty,
                        QTInner::Empty,
                    ],
                    alloc,
                ));
                sd._insert(
                    pos,
                    SpatialData {
                        data: data.clone(),
                        position: pos,
                    },
                    &alloc,
                );
                sd._insert(pos, value, &alloc);
                *self = sd;
            }
        }
    }
}

fn main() {
    // dbg!(size_of::<Quadtree<f32>>());

    // // let qt: Quadtree<_> = Quadtree::Value(0x69696969);
    // let qt: Quadtree<f32> = Quadtree::Subdivision(Box::new([Quadtree::Empty, Quadtree::Empty, Quadtree::Empty, Quadtree::Empty]));
    // // let qt: Quadtree<f32> = Quadtree::Empty;

    // let ptr: *const u128 = unsafe { std::mem::transmute(&qt) };
    // println!("{:x}", unsafe { ptr.read() });

    // return;

    const N: usize = 100;

    let values = (0..N)
        .flat_map(|x| {
            (0..N).map(move |y| SpatialData {
                position: (x as f32 / 100.0, y as f32 / 100.0),
                data: x * y,
            })
        })
        .collect::<Vec<SpatialData<usize>>>();

    let st = Instant::now();

    let bump = Bump::new();
    let mut qt = Quadtree::new_in(&bump);

    for value in values {
        qt.insert(value);
    }

    let et = Instant::now();

    println!("Time: {:?}", et.duration_since(st));
}
