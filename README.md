# str0m-benches

Benchmark tests for str0m using [divan](https://github.com/nvzqz/divan). 
The benches rely on the str0m "_internal_test_exports" feature.


## Examples

```
√ str0m-benches (git)-[master]- # cargo bench
     Running benches/bench0.rs (target/release/deps/bench0-9d00dcf40629382d)
Timer precision: 41 ns
bench0         fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ vp8_unidir  217.3 ms      │ 378.7 ms      │ 272.2 ms      │ 275.1 ms      │ 100     │ 100
╰─ vp9_unidir  691.9 ms      │ 871.3 ms      │ 745.9 ms      │ 751.3 ms      │ 100     │ 100


√ str0m-benches (git)-[master]- # cargo bench vp8 -F allocations
     Running benches/bench0.rs (target/release/deps/bench0-51629209bf181d93)
Timer precision: 41 ns
bench0         fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ vp8_unidir  228.6 ms      │ 469.2 ms      │ 278.8 ms      │ 286.1 ms      │ 100     │ 100
               alloc:        │               │               │               │         │
                 464161      │ 463873        │ 464161        │ 459468        │         │
                 253.7 MB    │ 253.3 MB      │ 253.7 MB      │ 251.1 MB      │         │
               dealloc:      │               │               │               │         │
                 464161      │ 463873        │ 464161        │ 459468        │         │
                 254.3 MB    │ 254 MB        │ 254.3 MB      │ 251.7 MB      │         │
               grow:         │               │               │               │         │
                 15975       │ 15879         │ 15975         │ 15798         │         │
                 688.2 KB    │ 685 KB        │ 688.2 KB      │ 680.8 KB      │         │
```

## Profiling Example

Benches provide a good way to profile at full cpu usage and in isolation (only str0m code, no io, sockets, threads)


```
EXECUTABLE=$(cargo bench --no-run 2>&1 | grep "Executable" | sed -n 's/.*(\([^)]*\)).*/\1/p')
sudo perf record --call-graph dwarf -F 2000 target/release/deps/bench0-9d00dcf40629382d --bench --profile-time 10
sudo perf script -f --no-inline > profile.perf
# -> profiler.firefox.com
```

   