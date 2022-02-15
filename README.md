<div align="center">
  <h1><code>bitsvec</code></h1>

  <p>
    <strong>A bit vector with <a href="https://github.com/rust-lang/portable-simd">the Rust standard library's portable SIMD API</a></strong>
  </p>

  <p>
    <a href="https://crates.io/crates/bitsvec"><img src="https://img.shields.io/crates/v/bitsvec.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/bitsvec"><img src="https://docs.rs/bitsvec/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

## Usage

Add `bitsvec` to `Cargo.toml`:

```toml
bitsvec = "x.y.z"
```

Write some code like this:

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

* [bit_vec 0.6.3](https://docs.rs/bit-vec/0.6.3/bit_vec/index.html)
* [bitvec 1.0.0](https://docs.rs/bitvec/1.0.0/bitvec/index.html)
* [bitvec_simd  0.15.0](https://docs.rs/bitvec_simd/0.15.0/bitvec_simd/index.html)
* [bitvector_simd 0.2.2](https://docs.rs/bitvector_simd/0.2.2/bitvector_simd/index.html)

<details open>

```
$ cargo bench

bitsvec(this crate)     time:   [330.34 ns 330.75 ns 331.18 ns]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

bitsvec_u16x8(this crate)
                        time:   [332.23 ns 333.90 ns 335.98 ns]
Found 16 outliers among 100 measurements (16.00%)
  6 (6.00%) high mild
  10 (10.00%) high severe

bitvec_simd 0.15.0      time:   [371.96 ns 372.23 ns 372.53 ns]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

bitvec_simd 0.15.0 u16x8
                        time:   [578.38 ns 578.68 ns 579.01 ns]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

bitvector_simd 0.2.2    time:   [288.55 ns 289.11 ns 289.64 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

bit-vec 0.6             time:   [1.5177 us 1.5200 us 1.5224 us]

bitvec 1.0              time:   [32.119 us 32.254 us 32.390 us]

bitsvec(this crate) with creation
                        time:   [888.59 ns 889.45 ns 890.39 ns]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) high mild
  1 (1.00%) high severe
                                                                                                              bitsvec_u16x8(this crate) with creation
                        time:   [1.1006 us 1.1031 us 1.1059 us]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe

bitvec_simd 0.15.0 with creation
                        time:   [970.09 ns 970.82 ns 971.67 ns]
Found 12 outliers among 100 measurements (12.00%)
  10 (10.00%) high mild
  2 (2.00%) high severe
                                                                                                              bitvec_simd 0.15.0 u16x8 with creation
                        time:   [1.1158 us 1.1185 us 1.1215 us]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) low mild
  2 (2.00%) high mild

bitvector_simd 0.2.2 with creation
                        time:   [736.46 ns 737.95 ns 739.49 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

bit-vec 0.6 with creation
                        time:   [1.6515 us 1.6527 us 1.6539 us]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

bitvec 1.0 with creation
                        time:   [28.484 us 28.501 us 28.518 us]
Found 20 outliers among 100 measurements (20.00%)
  15 (15.00%) low severe
  3 (3.00%) high mild
  2 (2.00%) high severe

bitsvec(this crate) resize false
                        time:   [676.30 ns 677.13 ns 677.93 ns]
                                                                                                              bitsvec_u16x8(this crate) resize false
                        time:   [618.70 ns 619.73 ns 620.98 ns]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

bitvec_simd 0.15.0 resize false
                        time:   [676.27 ns 677.96 ns 679.66 ns]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

bitvec_simd 0.15.0 u16x8 resize false
                        time:   [472.84 ns 473.76 ns 474.71 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

bitvec 1.0 resize false time:   [108.23 us 108.29 us 108.36 us]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

bitsvec(this crate) resize true
                        time:   [679.71 ns 680.66 ns 681.75 ns]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe

bitsvec_u16x8(this crate) resize true
                        time:   [876.21 ns 876.89 ns 877.71 ns]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe

bitvec_simd 0.15.0 resize true
                        time:   [672.44 ns 672.82 ns 673.24 ns]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

bitvec_simd 0.15.0 u16x8 resize true
                        time:   [748.77 ns 751.48 ns 754.59 ns]

bitvec 1.0 resize true  time:   [100.50 us 100.63 us 100.75 us]
```

</details open>

## Credits

Most code of this crate is from (https://github.com/GCCFeli/bitvec_simd). On top of that, some changes were made.

## License

This library is licensed under either of:

* MIT license [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT
* Apache License 2.0 [LICENSE-APACHE](LICENSE-APACHE) or https://opensource.org/licenses/Apache-2.0

at your option.
