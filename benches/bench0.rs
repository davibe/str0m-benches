// Add the necessary dependencies in your Cargo.toml file:

mod common;
use common::pair::LtoR;

use divan::{black_box, Bencher};

#[cfg(not(feature = "allocations"))]
use tikv_jemallocator::Jemalloc;
#[cfg(not(feature = "allocations"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[cfg(feature = "allocations")]
use divan::AllocProfiler;
#[cfg(feature = "allocations")]
#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

#[divan::bench(threads)]
pub fn vp8_unidir(bencher: Bencher) {
    let server = LtoR::with_vp8_input();
    let server_ref = &server;

    bencher.bench_local(|| {
       let _ = black_box(black_box(server_ref).run().expect("error")); 
    });
}

#[divan::bench(threads)]
pub fn vp9_unidir(bencher: Bencher) {
    let server = LtoR::with_vp9_input();
    let server_ref = &server;

    bencher.bench_local(|| {
       let _ = black_box(black_box(server_ref).run().expect("error")); 
    });
}

fn main() {
    divan::main();
}


