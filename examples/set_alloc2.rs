extern crate stats_alloc;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

use std::collections::{BTreeSet, HashSet};
use vec_collections::VecSet;

struct PlotStats {
    bytes_allocated: usize,
    bytes_persistent: usize,
    allocations: usize,
}

impl From<stats_alloc::Stats> for PlotStats {
    fn from(value: stats_alloc::Stats) -> Self {
        Self {
            bytes_allocated: value.bytes_allocated,
            bytes_persistent: value.bytes_allocated - value.bytes_deallocated,
            allocations: value.allocations,
        }
    }
}

fn main() {
    let n = 1000;
    let mut bs = Vec::<PlotStats>::new();
    for i in 0..n {
        let reg = Region::new(&GLOBAL);
        let r = (0..i).collect::<BTreeSet<_>>();
        let stats: stats_alloc::Stats = reg.change();
        bs.push(stats.into());
        std::mem::drop(r);
    }
    let mut vs = Vec::<PlotStats>::new();
    for i in 0..n {
        let reg = Region::new(&GLOBAL);
        let mut r = (0..i).collect::<VecSet<[u32; 4]>>();
        r.shrink_to_fit();
        let stats: stats_alloc::Stats = reg.change();
        vs.push(stats.into());
    }
    let mut hs = Vec::<PlotStats>::new();
    for i in 0..n {
        let reg = Region::new(&GLOBAL);
        let mut r = (0..i).collect::<HashSet<_>>();
        r.shrink_to_fit();
        let stats: stats_alloc::Stats = reg.change();
        hs.push(stats.into());
        std::mem::drop(r);
    }
    println!("n,\tBTreeSet alloc,\tBTreeSet persistent,\tBTreeSet nalloc,\tVecSet alloc,\tVecSet persistent,\tVecSet nalloc,\tHashSet alloc,\tHashSet persistent,\tHashSet nalloc");
    for i in 0..(n as usize) {
        println!(
            "{},\t{},\t{},\t{},\t{},\t{},\t{},\t{},\t{},\t{}",
            i,
            bs[i].bytes_allocated,
            bs[i].bytes_persistent,
            bs[i].allocations,
            vs[i].bytes_allocated,
            vs[i].bytes_persistent,
            vs[i].allocations,
            hs[i].bytes_allocated,
            hs[i].bytes_persistent,
            hs[i].allocations,
        );
    }
}
