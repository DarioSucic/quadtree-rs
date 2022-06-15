#![feature(allocator_api)]

use std::alloc::{Allocator, System};
use std::mem::forget;
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

    fn traverse(&self, mut f: impl FnMut(usize, &T)) {
        self.root._traverse(0, &mut f);
    }
}

impl <T: Clone> Quadtree<'static, T, System> {
    fn new() -> Self {
        Self {
            root: QTInner::Empty,
            storage: &System
        }
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

    fn _traverse(&self, depth: usize, f: &mut impl FnMut(usize, &T)) {
        match self {
            QTInner::Empty => (),
            QTInner::Value(v) => f(depth, v),
            QTInner::Subdivision(s) => s.iter().for_each(|qti| qti._traverse(depth+1, f)),
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

    println!("Page Size: {}", page_size::get());

    const N: usize = 1000;

    let mut values = Vec::with_capacity(N*N);
    for x in 0..N {
        for y in 0..N {
            values.push(SpatialData {
                position: (x as f32 / 100.0, y as f32 / 100.0),
                data: x * y,
            });
        }
    }

    let st = Instant::now();
    // let mut bump = Bump::new();
    let mut bump = Bump::with_capacity(values.len() *  size_of::<QTInner<usize, System>>());
    let mut qt = Quadtree::new_in(&bump);
    // let mut qt = Quadtree::new();
    for value in values {
        qt.insert(value);
    }
    let et = Instant::now();
    println!("Insert   :: {:>8.2?}", et.duration_since(st));

    let st = Instant::now();
    let mut sum = 0;
    qt.traverse(|depth, value| {
        sum += value;
    });
    let et = Instant::now();
    println!("Traverse :: {:>8.2?}", et.duration_since(st));

    let st = Instant::now();
    
    // Slow
    // drop(qt);
    // drop(bump);

    // Fast
    
    {
        forget(qt);
        // for chunk in bump.iter_allocated_chunks() {
        //     println!("{:?}", chunk.len());
        // }
        drop(bump);
    }
    

    // Slow
    
    {
        // drop(qt);
    }
    

    let et = Instant::now();
    println!("Drop     :: {:>8.2?}", et.duration_since(st));

    println!("Sum: {sum}");

}
