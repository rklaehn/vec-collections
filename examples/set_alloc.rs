extern crate jemalloc_ctl;
extern crate jemallocator;

use std::collections::{BTreeSet, HashSet};
use vec_collections::VecSet;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() -> jemalloc_ctl::Result<()> {
    let allocated = jemalloc_ctl::thread::allocatedp::mib()?;
    let allocated = allocated.read()?;
    let deallocated = jemalloc_ctl::thread::deallocatedp::mib()?;
    let deallocated = deallocated.read()?;
    let n = 1000;
    let mut bs = Vec::new();
    for i in 0..n {
        let a0 = allocated.get();
        let d0 = deallocated.get();
        let r = (0..i).collect::<BTreeSet<_>>();
        let a1 = allocated.get();
        let d1 = deallocated.get();
        let allocated = a1 - a0;
        let deallocated = d1 - d0;
        bs.push((allocated - deallocated, deallocated));
        // println!("{},\t{},\t{}", r.len(), a1 - a0, d1 - d0);
        std::mem::drop(r);
    }
    let mut vs = Vec::new();
    for i in 0..n {
        let a0 = allocated.get();
        let d0 = deallocated.get();
        let r = (0..i).collect::<VecSet<[u64; 16]>>();
        let a1 = allocated.get();
        let d1 = deallocated.get();
        let allocated = a1 - a0;
        let deallocated = d1 - d0;
        vs.push((allocated - deallocated, deallocated));
        // println!("{},\t{},\t{}", r.len(), a1 - a0, d1 - d0);
        std::mem::drop(r);
    }
    let mut hs = Vec::new();
    for i in 0..n {
        let a0 = allocated.get();
        let d0 = deallocated.get();
        let r = (0..i).collect::<HashSet<_>>();
        let a1 = allocated.get();
        let d1 = deallocated.get();
        let allocated = a1 - a0;
        let deallocated = d1 - d0;
        hs.push((allocated - deallocated, deallocated));
        // println!("{},\t{},\t{}", r.len(), a1 - a0, d1 - d0);
        std::mem::drop(r);
    }
    println!("n\tBTreeSet\tVecSet\tHashSet\tFnvHashSet");
    for i in 0..(n as usize) {
        println!(
            "{},\t{},\t{},\t{},\t{},\t{},\t{}",
            i, bs[i].0, bs[i].1, vs[i].0, vs[i].1, hs[i].0, hs[i].1
        );
    }
    Ok(())
}
