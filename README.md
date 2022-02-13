## bitsvec

A bit vector with [the Rust standard library's portable SIMD API](https://github.com/rust-lang/portable-simd).

## How to use

```rust
let mut bitvec = BitVec::ones(1000); // create a bitvec contains 0 ..= 999
bitvec.set(900, false); // delete 900 from bitvec
bitvec.set(1200, true); // add 1200 to bitvec (and expand bitvec to length 1201)
let bitvec2 = BitVec::ones(1000);

let new_bitvec = bitvec.and_cloned(&bitvec2); // and operation, without consume
let new_bitvec2 = bitvec & bitvec2; // and operation, consume both bitvec

// Operation Supported:
// and, or, xor, not, eq, eq_left

assert_eq!(new_bitvec, new_bitvec2);
```

## Performance

Compared on AMD Ryzen 9 5900hs, aginst:

* [bit\_vec 0.6.3](https://docs.rs/bit-vec/0.6.3/bit_vec/index.html)
* [bitvec 1.0.0](https://docs.rs/bitvec/1.0.0/bitvec/index.html)

```
$ cargo bench

bitsvec(this crate)     time:   [348.51 ns 348.74 ns 348.98 ns]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild

bitsvec_u16x8(this crate)
                        time:   [401.90 ns 403.02 ns 404.94 ns]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high severe

bit-vec 0.6             time:   [1.5891 us 1.5956 us 1.6034 us]
Found 13 outliers among 100 measurements (13.00%)
  6 (6.00%) high mild
  7 (7.00%) high severe

bitvec 1.0              time:   [32.228 us 32.516 us 32.821 us]
Found 16 outliers among 100 measurements (16.00%)
  13 (13.00%) low severe
  3 (3.00%) high mild

bitsvec(this crate) with creation
                        time:   [884.09 ns 884.70 ns 885.35 ns]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

bitsvec_u16x8(this crate) with creation
                        time:   [878.66 ns 879.61 ns 880.66 ns]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

bit-vec 0.6 with creation
                        time:   [495.50 ns 495.87 ns 496.27 ns]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

bitvec 1.0 with creation
                        time:   [29.003 us 29.028 us 29.058 us]
Found 15 outliers among 100 measurements (15.00%)
  5 (5.00%) low severe
  8 (8.00%) high mild
  2 (2.00%) high severe
```

## Credits

Most code of this crate is from (https://github.com/GCCFeli/bitvec_simd). On top of that, some changes were made.
