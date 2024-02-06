// Add the necessary dependencies in your Cargo.toml file:

mod common;
use common::pair::LtoR;

use divan::{black_box, Bencher};

use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

#[divan::bench(threads)]
pub fn vp8_unidir(bencher: Bencher) {
    let server = LtoR::with_vp8_input();
    let server_ref = &server;

    bencher.bench(|| {
       let _ = black_box(black_box(server_ref).run().expect("error")); 
    });
}

#[divan::bench(threads)]
pub fn vp9_unidir(bencher: Bencher) {
    let server = LtoR::with_vp9_input();
    let server_ref = &server;

    bencher.bench(|| {
       let _ = black_box(black_box(server_ref).run().expect("error")); 
    });
}

fn main() {
    divan::main();
}


