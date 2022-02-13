//! ## BitVec implemented with [portable-simd](https://github.com/rust-lang/portable-simd).
//!
//! BitVec represents numbers by the position of bits. For example, for the set $\{1,3,5\}$, we
//! can represent it by a just a byte `010101000` -- the most left (high) bit represent if `0`
//! exits in this set or not, the second bit represent `1` ...
//!
//! BitVec is usually used in the algorithm which requires many set intersection/union operations,
//! such like graph mining, formal concept analysis. Set operations in bitvec can be implemented
//! with simple and/or/xor operations so it is much faster than "normal" version of `HashSet`.
//!
//! Furthermore, as SIMD introduces the ability for handling multiple data with a single instruction,
//! set operations can be even faster with SIMD enabled.
//!
//! However, implementation with SIMD in Rust is not really an easy task -- now only low-level API
//! is provided through [core::arch](https://doc.rust-lang.org/core/arch/index.html). It requires
//! many `cfg(target_arch)`s (i.e. different implement on different arch) and
//! assembly-like unsafe function calls.
//!
//! The Rust standard library's portable SIMD API provided a much better API for users. You can just
//! treat SIMD operations as an operation on slices. Portable SIMD wraps all the low-level details
//! for you -- no arch-specified code, no unsafe, just do what you've done on normal integer/floats.
//!
//! This crate uses Portable SIMD to implement a basic bitvec.
//!
//! ### Usage
//!
//! ```rust
//! use bitsvec::BitVec;
//!
//! let mut bitvec = BitVec::ones(1_000); //create a set containing 0 ..= 999
//! bitvec.set(1_999, true); // add 1999 to the set, bitvec will be automatically expanded
//! bitvec.set(500, false); // delete 500 from the set
//! // now the set contains: 0 ..=499, 501..=1999
//! assert_eq!(bitvec.get(500), Some(false));
//! assert_eq!(bitvec.get(5_000), None);
//! // When try to get number larger than current bitvec, it will return `None`.
//! // of course if you don't care, you can just do:
//! assert_eq!(bitvec.get(5_000).unwrap_or(false), false);
//!
//! let bitvec2 = BitVec::zeros(2000); // create a set containing 0 ..=1999
//!
//! let bitvec3 = bitvec.and_cloned(&bitvec2);
//! // and/or/xor/not operation is provided.
//! // these APIs usually have 2 version:
//! // `.and` consume the inputs and `.and_clone()` accepts reference and will do clone on inputs.
//! let bitvec4 = bitvec & bitvec2;
//! // ofcourse you can just use bit-and operator on bitvecs, it will also consumes the inputs.
//! assert_eq!(bitvec3, bitvec4);
//! // A bitvec can also be constructed from a collection of bool, or a colluction of integer:
//! let bitvec: BitVec = (0 .. 10).map(|x| x%2 == 0).into();
//! let bitvec2: BitVec = (0 .. 10).map(|x| x%3 == 0).into();
//! let bitvec3 = BitVec::from_bool_iterator((0..10).map(|x| x%6 == 0));
//! assert_eq!(bitvec & bitvec2, bitvec3)
//! ```
//!
//! ## Performance
//!
//! run `cargo bench` to see the benchmarks on your device.
//! Benchmarks in author's environment show that this library has the advantage at `u16x8`
//! among the bitvector libraries currently implemented based on SIMD.
//! In addition, any SIMD bitvector implementation is inferior to [bit-vec](https://docs.rs/bit-vec/) in `creation`,
//! while this library performs relatively well overall.

#![feature(portable_simd)]
#![no_std]

#[cfg(any(test, feature = "std"))]
#[macro_use]
extern crate std;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::fmt::{Binary, Debug, Display, Formatter, Result};
use core::ops::*;
use core::simd::*;

use smallvec::{Array, SmallVec};

#[cfg(test)]
mod tests;

// Declare the default BitVec type
pub type BitVec = BitVecSimd<[u64x4; 4], 4>;

/// Representation of a BitVec
///
/// see the module's document for examples and details.
///
#[derive(Debug, Clone)]
#[repr(C)]
pub struct BitVecSimd<A, const L: usize>
where
    A: Array + Index<usize>,
    A::Item: BitContainer<L>,
{
    // internal representation of bitvec
    // TODO: try `Vec`, may be better than smallvec
    storage: SmallVec<A>,
    // actual number of bits exists in storage
    nbits: usize,
}

/// Proc macro can not export BitVec
/// macro_rules! can not cancot ident
/// so we use name, name_2 for function names
macro_rules! impl_operation {
    ($name:ident, $name_cloned:ident, $name_inplace:ident, $op:tt) => {
        pub fn $name(self, other: Self) -> Self {
            assert_eq!(self.nbits, other.nbits);
            let storage = self
                .storage
                .into_iter()
                .zip(other.storage.into_iter())
                .map(|(a, b)| a $op b)
                .collect();
            Self {
                storage,
                nbits: self.nbits,
            }
        }
        pub fn $name_cloned(&self, other: &Self) -> Self {
            assert_eq!(self.nbits, other.nbits);
            let storage = self
                .storage
                .iter()
                .cloned()
                .zip(other.storage.iter().cloned())
                .map(|(a, b)| a $op b)
                .collect();
            Self {
                storage,
                nbits: self.nbits,
            }
        }
        pub fn $name_inplace(&mut self, other: &Self) {
            assert_eq!(self.nbits, other.nbits);
            self.storage.iter_mut().zip(other.storage.iter()).for_each(|(a, b)| a.$name_inplace(b));
        }
    };
}

impl<A, const L: usize> BitVecSimd<A, L>
where
    A: Array + Index<usize>,
    A::Item: BitContainer<L>,
{
    // convert total bit to length
    // input: Number of bits
    // output:
    //
    // 1. the number of Vector used
    // 2. after filling 1, the remaining bytes should be filled
    // 3. after filling 2, the remaining bits should be filled
    //
    // notice that this result represents the length of vector
    // so if 3. is 0, it means no extra bits after filling bytes
    // return (length of storage, u64 of last container, bit of last elem)
    // any bits > length of last elem should be set to 0
    #[inline]
    fn bit_to_len(nbits: usize) -> (usize, usize, usize) {
        (
            nbits / (A::Item::BIT_WIDTH as usize),
            (nbits % (A::Item::BIT_WIDTH as usize)) / A::Item::ELEMENT_BIT_WIDTH,
            nbits % A::Item::ELEMENT_BIT_WIDTH,
        )
    }

    #[inline]
    fn set_bit(
        flag: bool,
        bytes: <A::Item as BitContainer<L>>::Element,
        offset: u32,
    ) -> <A::Item as BitContainer<L>>::Element {
        match flag {
            true => bytes | A::Item::ONE_ELEMENT.wrapping_shl(offset),
            false => bytes & !A::Item::ONE_ELEMENT.wrapping_shl(offset),
        }
    }

    /// Create an empty bitvec with `nbits` initial elements.
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec = BitVec::zeros(10);
    /// assert_eq!(bitvec.len(), 10);
    /// ```
    pub fn zeros(nbits: usize) -> Self {
        let len = (nbits + A::Item::BIT_WIDTH - 1) / A::Item::BIT_WIDTH;
        let storage = (0..len).map(|_| A::Item::ZERO).collect();
        Self { storage, nbits }
    }

    /// Create a bitvec containing all 0 .. nbits elements.
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec = BitVec::ones(10);
    /// assert_eq!(bitvec.len(), 10);
    /// ```
    pub fn ones(nbits: usize) -> Self {
        let (len, bytes, bits) = Self::bit_to_len(nbits);
        let mut storage = (0..len).map(|_| A::Item::MAX).collect::<SmallVec<_>>();
        if bytes > 0 || bits > 0 {
            let mut arr = A::Item::MAX.to_array();
            arr[bytes] =
                A::Item::MAX_ELEMENT.clear_high_bits((A::Item::ELEMENT_BIT_WIDTH - bits) as u32);
            for item in arr.iter_mut().take(A::Item::LANES).skip(bytes + 1) {
                *item = A::Item::ZERO_ELEMENT;
            }
            storage.push(A::Item::from(arr));
        }
        Self { storage, nbits }
    }

    /// Create a bitvec from an Iterator of bool.
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec = BitVec::from_bool_iterator((0..10).map(|x| x % 2 == 0));
    /// assert_eq!(bitvec.len(), 10);
    /// assert_eq!(<BitVec as Into<Vec<bool>>>::into(bitvec), vec![true, false, true, false, true, false, true, false, true, false]);
    ///
    /// let bitvec = BitVec::from_bool_iterator((0..1000).map(|x| x < 50));
    /// assert_eq!(bitvec.len(), 1000);
    /// assert_eq!(bitvec.get(49), Some(true));
    /// assert_eq!(bitvec.get(50), Some(false));
    /// assert_eq!(bitvec.get(999), Some(false));
    /// assert_eq!(<BitVec as Into<Vec<bool>>>::into(bitvec), (0..1000).map(|x| x<50).collect::<Vec<bool>>());
    /// ```
    pub fn from_bool_iterator<I: Iterator<Item = bool>>(i: I) -> Self {
        // FIXME: any better implementation?
        let mut storage = SmallVec::new();
        let mut current_slice = A::Item::ZERO.to_array();
        let mut nbits = 0;
        for b in i {
            if b {
                current_slice[nbits % A::Item::BIT_WIDTH / A::Item::ELEMENT_BIT_WIDTH] |=
                    A::Item::ONE_ELEMENT.wrapping_shl((nbits % A::Item::ELEMENT_BIT_WIDTH) as u32);
            }
            nbits += 1;
            if nbits % A::Item::BIT_WIDTH == 0 {
                storage.push(A::Item::from(current_slice));
                current_slice = A::Item::ZERO.to_array();
            }
        }
        if nbits % A::Item::BIT_WIDTH > 0 {
            storage.push(A::Item::from(current_slice));
        }
        Self { storage, nbits }
    }

    /// Initialize from a set of integers.
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec = BitVec::from_slice(&[0,5,9]);
    /// assert_eq!(<BitVec as Into<Vec<bool>>>::into(bitvec), vec![true, false, false, false, false, true, false, false, false, true]);
    /// ```
    pub fn from_slice(slice: &[usize]) -> Self {
        let mut bv = BitVecSimd::zeros(slice.len());
        for i in slice {
            bv.set(*i, true);
        }
        bv
    }

    /// Initialize from a E slice.
    /// Data will be copied from the slice.
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec = BitVec::from_slice_copy(&[3], 3);
    /// assert_eq!(bitvec.get(0), Some(true));
    /// assert_eq!(bitvec.get(1), Some(true));
    /// assert_eq!(bitvec.get(2), Some(false));
    /// assert_eq!(bitvec.get(3), None);
    /// ```
    pub fn from_slice_copy(slice: &[<A::Item as BitContainer<L>>::Element], nbits: usize) -> Self {
        let len = (nbits + A::Item::ELEMENT_BIT_WIDTH - 1) / A::Item::ELEMENT_BIT_WIDTH;
        assert!(len <= slice.len());

        let iter = &mut slice.iter();
        let mut storage = SmallVec::with_capacity((len + A::Item::LANES - 1) / A::Item::LANES);
        let (i, bytes, bits) = Self::bit_to_len(nbits);

        while let Some(a0) = iter.next() {
            let mut arr = A::Item::ZERO.to_array();
            arr[0] = *a0;
            for item in arr.iter_mut().take(A::Item::LANES).skip(1) {
                *item = *(iter.next().unwrap_or(&A::Item::ZERO_ELEMENT));
            }

            if storage.len() == i && (bytes > 0 || bits > 0) {
                Self::clear_arr_high_bits(&mut arr, bytes, bits);
            }
            storage.push(A::Item::from(arr));
        }

        Self { storage, nbits }
    }

    /// Initialize from a raw buffer.
    /// Data will be copied from the buffer which [ptr] points to.
    /// The buffer can be released after initialization.
    ///
    /// # Safety
    ///
    /// If any of the following conditions are violated, the result is Undefined
    /// Behavior:
    ///
    /// * ptr should be valid and point to an [allocated object] with length >= buffer_len
    ///
    /// * ptr.offset(buffer_len - 1), **in bytes**, cannot overflow an `isize`.
    ///
    /// * The offset being in bounds cannot rely on "wrapping around" the address
    ///   space. That is, the infinite-precision sum, **in bytes** must fit in a usize.
    ///
    pub unsafe fn from_raw_copy(
        ptr: *const <A::Item as BitContainer<L>>::Element,
        buffer_len: usize,
        nbits: usize,
    ) -> Self {
        let len = (nbits + A::Item::ELEMENT_BIT_WIDTH - 1) / A::Item::ELEMENT_BIT_WIDTH;
        assert!(len <= buffer_len);

        let mut storage = SmallVec::with_capacity((len + A::Item::LANES - 1) / A::Item::LANES);
        let (i, bytes, bits) = Self::bit_to_len(nbits);

        for index in 0..(len as isize) {
            let mut arr = A::Item::ZERO.to_array();
            for (j, item) in arr.iter_mut().enumerate().take(A::Item::LANES) {
                let k = index * A::Item::LANES as isize + j as isize;
                *item = if k < len as isize {
                    // The only unsafe operation happens here
                    *(ptr.offset(k))
                } else {
                    A::Item::ZERO_ELEMENT
                };
            }
            if storage.len() == i && (bytes > 0 || bits > 0) {
                Self::clear_arr_high_bits(&mut arr, bytes, bits);
            }
            storage.push(A::Item::from(arr));
        }

        Self { storage, nbits }
    }

    /// Length of this bitvec.
    ///
    /// To get the number of elements, use `count_ones`
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec = BitVec::ones(3);
    /// assert_eq!(bitvec.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.nbits
    }

    /// Length of underlining storage.
    #[inline]
    pub fn storage_len(&self) -> usize {
        self.storage.len()
    }

    /// Capacity of underlining storage.
    #[inline]
    pub fn storage_capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Returns a raw pointer to the vector's buffer.
    pub fn as_ptr(&self) -> *const A::Item {
        self.storage.as_ptr()
    }

    /// Returns a raw mutable pointer to the vector's buffer.
    pub fn as_mut_ptr(&mut self) -> *mut A::Item {
        self.storage.as_mut_ptr()
    }

    /// Returns `true` if the data has spilled into a separate heap-allocated buffer.
    #[inline]
    pub fn spilled(&self) -> bool {
        self.storage.spilled()
    }

    fn clear_arr_high_bits(
        arr: &mut [<A::Item as BitContainer<L>>::Element],
        bytes: usize,
        bits: usize,
    ) {
        let mut end_bytes = bytes;
        if bits > 0 {
            arr[end_bytes] =
                arr[end_bytes].clear_high_bits((A::Item::ELEMENT_BIT_WIDTH - bits) as u32);
            end_bytes += 1;
        }
        for item in arr.iter_mut().take(A::Item::LANES).skip(end_bytes) {
            *item = A::Item::ZERO_ELEMENT;
        }
    }

    fn fill_arr_high_bits(
        arr: &mut [<A::Item as BitContainer<L>>::Element],
        bytes: usize,
        bits: usize,
        bytes_max: usize,
    ) {
        let mut end_bytes = bytes;
        if bits > 0 {
            arr[end_bytes] |= A::Item::MAX_ELEMENT.clear_low_bits(bits as u32);
            end_bytes += 1;
        }
        for item in arr.iter_mut().take(bytes_max).skip(end_bytes) {
            *item = A::Item::MAX_ELEMENT;
        }
    }

    fn clear_high_bits(&mut self, i: usize, bytes: usize, bits: usize) {
        if bytes > 0 || bits > 0 {
            let mut arr = self.storage[i].to_array();
            Self::clear_arr_high_bits(&mut arr, bytes, bits);
            self.storage[i] = A::Item::from(arr);
        }
    }

    fn fill_high_bits(&mut self, i: usize, bytes: usize, bits: usize, bytes_max: usize) {
        if bytes > 0 || bits > 0 {
            let mut arr = self.storage[i].to_array();
            Self::fill_arr_high_bits(&mut arr, bytes, bits, bytes_max);
            self.storage[i] = A::Item::from(arr);
        }
    }

    fn fix_high_bits(
        &mut self,
        old_i: usize,
        old_bytes: usize,
        old_bits: usize,
        i: usize,
        bytes: usize,
        bits: usize,
    ) {
        debug_assert!(old_i == i && old_bytes <= bytes && (bytes > 0 || bits > 0));
        let mut arr = self.storage[i].to_array();
        if old_bytes < bytes {
            Self::fill_arr_high_bits(
                &mut arr,
                old_bytes,
                old_bits,
                if bits > 0 { bytes + 1 } else { bytes },
            );
        } else {
            debug_assert!(old_bytes == bytes && bits >= old_bits);
            if bits > old_bits {
                // fix the only byte
                arr[bytes] |= A::Item::MAX_ELEMENT.clear_low_bits(old_bits as u32);
            }
        }
        Self::clear_arr_high_bits(&mut arr, bytes, bits);
        self.storage[i] = A::Item::from(arr);
    }

    /// Resize this bitvec to `nbits` in-place.
    /// If new length is greater than current length, `value` will be filled.
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let mut bitvec = BitVec::ones(3);
    /// bitvec.resize(5, false);
    /// assert_eq!(bitvec.len(), 5);
    /// bitvec.resize(2, false);
    /// assert_eq!(bitvec.len(), 2);
    /// ```
    pub fn resize(&mut self, nbits: usize, value: bool) {
        let (i, bytes, bits) = Self::bit_to_len(nbits);
        self.storage.resize(
            if bytes > 0 || bits > 0 { i + 1 } else { i },
            if value { A::Item::MAX } else { A::Item::ZERO },
        );
        if nbits < self.nbits {
            self.clear_high_bits(i, bytes, bits);
        } else if value {
            // old_i <= i && filling 1
            let (old_i, old_bytes, old_bits) = Self::bit_to_len(self.nbits);
            if old_i < i {
                self.fill_high_bits(old_i, old_bytes, old_bits, A::Item::LANES);
                self.clear_high_bits(i, bytes, bits);
            } else if bytes > 0 || bits > 0 {
                self.fix_high_bits(old_i, old_bytes, old_bits, i, bytes, bits);
            }
        }
        self.nbits = nbits;
    }

    /// Shink this bitvec to new length in-place.
    /// Panics if new length is greater than original.
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let mut bitvec = BitVec::ones(3);
    /// bitvec.shrink_to(2);
    /// assert_eq!(bitvec.len(), 2);
    /// ```
    pub fn shrink_to(&mut self, nbits: usize) {
        if nbits >= self.nbits {
            panic!(
                "nbits {} should be less than current value {}",
                nbits, self.nbits
            );
        }
        self.resize(nbits, false);
    }

    /// Remove or add `index` to the set.
    /// If index > self.len, the bitvec will be expanded to `index`.
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let mut bitvec = BitVec::zeros(10);
    /// assert_eq!(bitvec.len(), 10);
    /// bitvec.set(15, true);
    /// // now 15 has been added to the set, its total len is 16.
    /// assert_eq!(bitvec.len(), 16);
    /// assert_eq!(bitvec.get(15), Some(true));
    /// assert_eq!(bitvec.get(14), Some(false));
    /// ```
    pub fn set(&mut self, index: usize, flag: bool) {
        let (i, bytes, bits) = Self::bit_to_len(index);
        if self.nbits <= index {
            let new_len = if bytes > 0 || bits > 0 { i + 1 } else { i };
            self.storage
                .extend((0..new_len - self.storage.len()).map(move |_| A::Item::ZERO));
            self.nbits = index + 1;
        }
        let mut arr = self.storage[i].to_array();
        arr[bytes] = Self::set_bit(flag, arr[bytes], bits as u32);
        self.storage[i] = A::Item::from(arr);
    }

    /// Copy content which ptr points to bitvec storage
    ///
    /// # Safety
    ///
    /// This is highly unsafe, due to the number of invariants that aren’t checked.
    pub unsafe fn set_raw_copy(&mut self, ptr: *mut A::Item, buffer_len: usize, nbits: usize) {
        let new_len = (nbits + A::Item::BIT_WIDTH - 1) / A::Item::BIT_WIDTH;
        assert!(new_len <= buffer_len);

        if new_len > self.len() {
            self.storage
                .extend((0..new_len - self.storage.len()).map(move |_| A::Item::ZERO));
        }

        for i in 0..(new_len as isize) {
            self.storage[i as usize] = *ptr.offset(i);
        }
        self.nbits = nbits;
    }

    /// Directly set storage to ptr
    ///
    /// # Safety
    ///
    /// This is highly unsafe, due to the number of invariants that aren’t checked.
    pub unsafe fn set_raw(
        &mut self,
        ptr: *mut A::Item,
        buffer_len: usize,
        capacity: usize,
        nbits: usize,
    ) {
        self.storage = SmallVec::from_raw_parts(ptr, buffer_len, capacity);
        self.nbits = nbits;
    }

    /// Set all items in bitvec to false
    pub fn set_all_false(&mut self) {
        self.storage
            .iter_mut()
            .for_each(move |x| *x = A::Item::ZERO);
    }

    /// Set all items in bitvec to true
    pub fn set_all_true(&mut self) {
        let (_, bytes, bits) = Self::bit_to_len(self.nbits);
        self.storage.iter_mut().for_each(move |x| *x = A::Item::MAX);
        if bytes > 0 || bits > 0 {
            let mut arr = A::Item::MAX.to_array();
            arr[bytes] =
                A::Item::MAX_ELEMENT.clear_high_bits((A::Item::ELEMENT_BIT_WIDTH - bits) as u32);
            for item in arr.iter_mut().take(A::Item::LANES).skip(bytes + 1) {
                *item = A::Item::ZERO_ELEMENT;
            }
            // unwrap here is safe since bytes > 0 || bits > 0 => self.nbits > 0
            *(self.storage.last_mut().unwrap()) = A::Item::from(arr);
        }
    }

    /// Set all items in bitvec to flag
    pub fn set_all(&mut self, flag: bool) {
        match flag {
            true => self.set_all_true(),
            false => self.set_all_false(),
        }
    }

    /// Check if `index` exists in current set.
    ///
    /// * If exists, return `Some(true)`
    /// * If index < current.len and element doesn't exist, return `Some(false)`.
    /// * If index >= current.len, return `None`.
    ///
    /// Examlpe:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec : BitVec = (0 .. 15).map(|x| x%3 == 0).into();
    /// assert_eq!(bitvec.get(3), Some(true));
    /// assert_eq!(bitvec.get(5), Some(false));
    /// assert_eq!(bitvec.get(14), Some(false));
    /// assert_eq!(bitvec.get(15), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<bool> {
        if self.nbits <= index {
            None
        } else {
            let (index, bytes, bits) = Self::bit_to_len(index);
            Some(
                self.storage[index].to_array()[bytes]
                    & A::Item::ONE_ELEMENT.wrapping_shl(bits as u32)
                    != A::Item::ZERO_ELEMENT,
            )
        }
    }

    /// Directly return a `bool` instead of an `Option`
    ///
    /// * If exists, return `true`.
    /// * If doesn't exist, return false.
    /// * If index >= current.len, panic.
    ///
    ///
    /// Examlpe:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec : BitVec = (0 .. 15).map(|x| x%3 == 0).into();
    /// assert_eq!(bitvec.get_unchecked(3), true);
    /// assert_eq!(bitvec.get_unchecked(5), false);
    /// assert_eq!(bitvec.get_unchecked(14), false);
    /// ```
    pub fn get_unchecked(&self, index: usize) -> bool {
        if self.nbits <= index {
            panic!("index out of bounds {} > {}", index, self.nbits);
        } else {
            let (index, bytes, bits) = Self::bit_to_len(index);
            (self.storage[index].to_array()[bytes] & A::Item::ONE_ELEMENT.wrapping_shl(bits as u32))
                != A::Item::ZERO_ELEMENT
        }
    }

    impl_operation!(and, and_cloned, and_inplace, &);
    impl_operation!(or, or_cloned, or_inplace, |);
    impl_operation!(xor, xor_cloned, xor_inplace, ^);

    /// difference operation
    ///
    /// `A.difference(B)` calculates `A\B`, e.g.
    ///
    /// ```text
    /// A = [1,2,3], B = [2,4,5]
    /// A\B = [1,3]
    /// ```
    ///
    /// also notice that
    ///
    /// ```text
    /// A.difference(B) | B.difference(A) == A ^ B
    /// ```
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec: BitVec = (0 .. 5_000).map(|x| x % 2 == 0).into();
    /// let bitvec2 : BitVec = (0 .. 5_000).map(|x| x % 3 == 0).into();
    /// assert_eq!(bitvec.difference_cloned(&bitvec2) | bitvec2.difference_cloned(&bitvec), bitvec.xor_cloned(&bitvec2));
    /// let bitvec3 : BitVec = (0 .. 5_000).map(|x| x % 2 == 0 && x % 3 != 0).into();
    /// assert_eq!(bitvec.difference(bitvec2), bitvec3);
    /// ```
    pub fn difference(self, other: Self) -> Self {
        self.and(other.not())
    }

    pub fn difference_cloned(&self, other: &Self) -> Self {
        // FIXME: This implementation has one extra clone
        self.and_cloned(&<&BitVecSimd<A, L>>::clone(&other).not())
    }

    // not should make sure bits > nbits is 0
    /// inverse every bits in the vector.
    ///
    /// If your bitvec have len `1_000` and contains `[1,5]`,
    /// after inverse it will contains `0, 2..=4, 6..=999`
    pub fn inverse(&self) -> Self {
        let (i, bytes, bits) = Self::bit_to_len(self.nbits);
        let mut storage = self.storage.iter().map(|x| !(*x)).collect::<SmallVec<_>>();
        if bytes > 0 || bits > 0 {
            assert_eq!(storage.len(), i + 1);
            let s: &mut A::Item = &mut storage[i];
            let mut arr = s.to_array();
            arr[bytes] = arr[bytes].clear_high_bits((A::Item::ELEMENT_BIT_WIDTH - bits) as u32);
            for item in arr.iter_mut().take(A::Item::LANES).skip(bytes + 1) {
                *item = A::Item::ZERO_ELEMENT;
            }
            *s = arr.into();
        }

        Self {
            storage,
            nbits: self.nbits,
        }
    }

    /// Count the number of elements existing in this bitvec.
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec: BitVec = (0..10_000).map(|x| x%2==0).into();
    /// assert_eq!(bitvec.count_ones(), 5000);
    ///
    /// let bitvec: BitVec = (0..30_000).map(|x| x%3==0).into();
    /// assert_eq!(bitvec.count_ones(), 10_000);
    /// ```
    pub fn count_ones(&self) -> usize {
        self.storage
            .iter()
            .map(|x| {
                x.to_array()
                    .into_iter()
                    .map(|a| a.count_ones())
                    .sum::<u32>()
            })
            .sum::<u32>() as usize
    }

    /// Count the number of elements existing in this bitvec, before the specified index.
    /// Panics if index is invalid.
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec: BitVec = (0..10_000).map(|x| x%2==0).into();
    /// assert_eq!(bitvec.count_ones_before(5000), 2500);
    ///
    /// let bitvec: BitVec = (0..30_000).map(|x| x%3==0).into();
    /// assert_eq!(bitvec.count_ones_before(10000), 3334);
    /// ```
    pub fn count_ones_before(&self, index: usize) -> usize {
        assert!(index <= self.nbits);
        if index == 0 {
            return 0;
        }
        let (i, bytes, bits) = Self::bit_to_len(index - 1);
        let mut ones = self
            .storage
            .iter()
            .take(i)
            .map(|x| {
                x.to_array()
                    .into_iter()
                    .map(|a| a.count_ones())
                    .sum::<u32>()
            })
            .sum::<u32>();
        if bytes > 0 || bits > 0 {
            // Safe unwrap here
            let arr = self.storage.iter().nth(i).unwrap().to_array();
            ones += arr
                .into_iter()
                .take(bytes)
                .map(|x| x.count_ones())
                .sum::<u32>();
            if bits > 0 {
                let x = arr.into_iter().nth(bytes).unwrap();
                ones += (x
                    & (A::Item::ONE_ELEMENT.wrapping_shl((bits + 1) as u32)
                        - A::Item::ONE_ELEMENT))
                    .count_ones();
            }
        }
        ones as usize
    }

    /// Count the number of leading zeros in this bitvec.
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let mut bitvec = BitVec::zeros(10);
    /// bitvec.set(3, true);
    /// assert_eq!(bitvec.leading_zeros(), 6);
    /// ```
    pub fn leading_zeros(&self) -> usize {
        let mut zero_item_count = 0;
        let mut iter = self
            .storage
            .iter()
            .rev()
            .skip_while(|x| match **x == A::Item::ZERO {
                true => {
                    zero_item_count += A::Item::LANES;
                    true
                }
                false => false,
            });

        if let Some(x) = iter.next() {
            let arr = x.to_array();
            let mut x_iter =
                arr.into_iter()
                    .rev()
                    .skip_while(|y| match *y == A::Item::ZERO_ELEMENT {
                        true => {
                            zero_item_count += 1;
                            true
                        }
                        false => false,
                    });

            // Safe unwrap here, since there should be at least one non-zero item in arr.
            let y = x_iter.next().unwrap();
            let raw_leading_zeros =
                zero_item_count * A::Item::ELEMENT_BIT_WIDTH + y.leading_zeros() as usize;
            let mut extra_leading_zeros = self.nbits % A::Item::BIT_WIDTH;
            if extra_leading_zeros > 0 {
                extra_leading_zeros = A::Item::BIT_WIDTH - extra_leading_zeros
            }
            return raw_leading_zeros as usize - extra_leading_zeros;
        }

        self.nbits
    }

    /// return true if contains at least 1 element
    pub fn any(&self) -> bool {
        self.storage.iter().any(|x| {
            x.to_array()
                .into_iter()
                .map(|a| a.count_ones())
                .sum::<u32>()
                > 0
        })
    }

    /// return true if contains self.len elements
    pub fn all(&self) -> bool {
        self.count_ones() == self.nbits
    }

    /// return true if set is empty
    pub fn none(&self) -> bool {
        !self.any()
    }

    /// Return true if set is empty.
    /// Totally the same with `self.none()`
    pub fn is_empty(&self) -> bool {
        !self.any()
    }

    /// Consume self and generate a `Vec<bool>` with length == self.len().
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec = BitVec::from_bool_iterator((0..10).map(|i| i % 3 == 0));
    /// let bool_vec = bitvec.into_bools();
    /// assert_eq!(bool_vec, vec![true, false, false, true, false, false, true, false, false, true])
    /// ```
    pub fn into_bools(self) -> Vec<bool> {
        self.into()
    }

    /// Consume self and geterate a `Vec<usize>` which only contains the number exists in this set.
    ///
    /// Example:
    ///
    /// ```rust
    /// use bitsvec::BitVec;
    ///
    /// let bitvec = BitVec::from_bool_iterator((0..10).map(|i| i%3 == 0));
    /// let usize_vec = bitvec.into_usizes();
    /// assert_eq!(usize_vec, vec![0,3,6,9]);
    /// ```
    pub fn into_usizes(self) -> Vec<usize> {
        self.into()
    }
}

impl<A, I: Iterator<Item = bool>, const L: usize> From<I> for BitVecSimd<A, L>
where
    A: Array + Index<usize>,
    A::Item: BitContainer<L>,
{
    fn from(i: I) -> Self {
        Self::from_bool_iterator(i)
    }
}

macro_rules! impl_trait {
    (
        ( $( $name:tt )+ ),
        ( $( $name1:tt )+ ),
        { $( $body:tt )* }
    ) =>
    {
        impl<A, const L: usize> $( $name )+ for $( $name1 )+
        where
            A: Array + Index<usize>,
            A::Item: BitContainer<L>,
        { $( $body )* }
    };
}

impl_trait! {
    (From< BitVecSimd<A, L> >),
    (Vec<bool>),
{
    fn from(v: BitVecSimd<A, L>) -> Self {
        v.storage
            .into_iter()
            .flat_map(|x| x.to_array())
                .flat_map(|x| {
                    (0..A::Item::ELEMENT_BIT_WIDTH)
                        .map(move |i| (x.wrapping_shr(i as u32)) & A::Item::ONE_ELEMENT != A::Item::ZERO_ELEMENT)
                })
            .take(v.nbits)
            .collect()
    }
}
}

impl_trait! {
    (From< BitVecSimd<A, L> >),
    (Vec<usize>),
{
    fn from(v: BitVecSimd<A, L>) -> Self {
        v.storage
            .into_iter()
            .flat_map(|x| x.to_array())
            .flat_map(|x| { (0..A::Item::ELEMENT_BIT_WIDTH).map(move |i| (x.wrapping_shr(i as u32)) & A::Item::ONE_ELEMENT != A::Item::ZERO_ELEMENT) })
            .take(v.nbits)
            .enumerate()
            .filter(|(_, b)| *b)
            .map(|(i, _)| i)
            .collect()
    }
}
}

impl_trait! {
    (Index<usize>),
    (BitVecSimd<A, L>),
    {
        type Output = bool;
        fn index(&self, index: usize) -> &Self::Output {
            if self.get_unchecked(index) {
                &true
            } else {
                &false
            }
        }
    }
}

impl_trait! {
    (Display),
    (BitVecSimd<A, L>),
    {
        fn fmt(&self, f: &mut Formatter) -> Result {
            for i in 0..self.nbits {
                write!(f, "{}", if self.get_unchecked(i) { 1 } else { 0 })?;
            }
            Ok(())
        }
    }
}

macro_rules! impl_eq_fn {
    ($( $rhs:tt )+) => {
        // eq should always ignore the bits > nbits
        fn eq(&self, other: $( $rhs )+) -> bool {
            assert_eq!(self.nbits, other.nbits);
            self.storage
                .iter()
                .zip(other.storage.iter())
                .all(|(a, b)| a == b)
        }
    }
}

impl_trait! { (PartialEq), (BitVecSimd<A, L>), { impl_eq_fn!(&Self); } }
impl_trait! { (PartialEq< &BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_eq_fn!(&&Self); } }
impl_trait! { (PartialEq< &mut BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_eq_fn!(&&mut Self); } }
impl_trait! { (PartialEq< BitVecSimd<A, L> >), (&BitVecSimd<A, L>), { impl_eq_fn!(&BitVecSimd<A, L>); } }
impl_trait! { (PartialEq< BitVecSimd<A, L> >), (&mut BitVecSimd<A, L>), { impl_eq_fn!(&BitVecSimd<A, L>); } }

macro_rules! impl_bit_op_fn {
    ($fn:ident, $op:ident, ( $( $rhs:tt )+ )) =>
    {
        type Output = BitVecSimd<A, L>;
        fn $fn(self, rhs: $( $rhs )+) -> Self::Output {
            self.$op(rhs)
        }
    };
    ($fn:ident, $op:ident, &, ( $( $rhs:tt )+ )) =>
    {
        type Output = BitVecSimd<A, L>;
        fn $fn(self, rhs: $( $rhs )+) -> Self::Output {
            self.$op(&rhs)
        }
    }
}

macro_rules! impl_bit_op {
    ($trait:ident, $fn:ident, $op:ident, $op_cloned:ident) => {
        impl_trait! {($trait), (BitVecSimd<A, L>), { impl_bit_op_fn!($fn, $op, (Self)); } } // a & b
        impl_trait! {($trait< &BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_bit_op_fn!($fn, $op_cloned, (&Self)); } } // a & &b
        impl_trait! { ($trait< &mut BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_bit_op_fn!($fn, $op_cloned, (&mut Self)); } } // a & &mut b
        impl_trait! { ($trait< BitVecSimd<A, L> >), (&BitVecSimd<A, L>), { impl_bit_op_fn!($fn, $op_cloned, &, (BitVecSimd<A, L>)); } } // &a & b
        impl_trait! { ($trait), (&BitVecSimd<A, L>), { impl_bit_op_fn!($fn, $op_cloned, (Self)); } } // &a & &b
        impl_trait! { ($trait< &mut BitVecSimd<A, L> >), (&BitVecSimd<A, L>), { impl_bit_op_fn!($fn, $op_cloned, (&mut BitVecSimd<A, L>)); } } // &a & &mut b
        impl_trait! { ($trait< BitVecSimd<A, L> >), (&mut BitVecSimd<A, L>), { impl_bit_op_fn!($fn, $op_cloned, &, (BitVecSimd<A, L>)); } } // &mut a & b
        impl_trait! { ($trait< &BitVecSimd<A, L> >), (&mut BitVecSimd<A, L>), { impl_bit_op_fn!($fn, $op_cloned, (&BitVecSimd<A, L>)); } } // &mut a & &b
        impl_trait! { ($trait), (&mut BitVecSimd<A, L>), { impl_bit_op_fn!($fn, $op_cloned, (Self)); } } // &mut a & &mut b
    };
}

impl_bit_op!(BitAnd, bitand, and, and_cloned);
impl_bit_op!(BitOr, bitor, or, or_cloned);
impl_bit_op!(BitXor, bitxor, xor, xor_cloned);

macro_rules! impl_not_fn {
    () => {
        type Output = BitVecSimd<A, L>;
        fn not(self) -> Self::Output {
            self.inverse()
        }
    };
}

impl_trait! {(Not), (BitVecSimd<A, L>), { impl_not_fn!(); }}
impl_trait! {(Not), (&BitVecSimd<A, L>), { impl_not_fn!(); }}
impl_trait! {(Not), (&mut BitVecSimd<A, L>), { impl_not_fn!(); }}

macro_rules! impl_bit_assign_fn {
    (($( $rhs:tt )+), $fn:ident, $fn1:ident, &) => {
        fn $fn(&mut self, rhs: $( $rhs )+) {
            self.$fn1(&rhs);
        }
    };
    (($( $rhs:tt )+), $fn:ident, $fn1:ident) => {
        fn $fn(&mut self, rhs: $( $rhs )+) {
            self.$fn1(rhs);
        }
    }
}

impl_trait! {(BitAndAssign), (BitVecSimd<A, L>), { impl_bit_assign_fn!((Self), bitand_assign, and_inplace, &); } }
impl_trait! {(BitAndAssign< &BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_bit_assign_fn!((&BitVecSimd<A, L>), bitand_assign, and_inplace); } }
impl_trait! {(BitAndAssign< &mut BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_bit_assign_fn!((&mut BitVecSimd<A, L>), bitand_assign, and_inplace); } }
impl_trait! {(BitOrAssign), (BitVecSimd<A, L>), { impl_bit_assign_fn!((Self), bitor_assign, or_inplace, &); } }
impl_trait! {(BitOrAssign< &BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_bit_assign_fn!((&BitVecSimd<A, L>), bitor_assign, or_inplace); } }
impl_trait! {(BitOrAssign< &mut BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_bit_assign_fn!((&mut BitVecSimd<A, L>), bitor_assign, or_inplace); } }
impl_trait! {(BitXorAssign), (BitVecSimd<A, L>), { impl_bit_assign_fn!((Self), bitxor_assign, xor_inplace, &); } }
impl_trait! {(BitXorAssign< &BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_bit_assign_fn!((&BitVecSimd<A, L>), bitxor_assign, xor_inplace); } }
impl_trait! {(BitXorAssign< &mut BitVecSimd<A, L> >), (BitVecSimd<A, L>), { impl_bit_assign_fn!((&mut BitVecSimd<A, L>), bitxor_assign, xor_inplace); } }

// BitContainerItem is the element of a SIMD type BitContainer
pub trait BitContainerItem:
    Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + BitAndAssign
    + BitOrAssign
    + Add<Output = Self>
    + Sub<Output = Self>
    + PartialEq
    + Sized
    + Copy
    + Clone
    + Binary
{
    const BIT_WIDTH: usize;
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;

    fn count_ones(self) -> u32;
    fn leading_zeros(self) -> u32;
    fn wrapping_shl(self, rhs: u32) -> Self;
    fn wrapping_shr(self, rhs: u32) -> Self;
    fn clear_high_bits(self, rhs: u32) -> Self;
    fn clear_low_bits(self, rhs: u32) -> Self;
}

macro_rules! impl_bitcontaineritem {
    ($type: ty, $zero: expr, $one: expr, $max: expr) => {
        impl BitContainerItem for $type {
            const BIT_WIDTH: usize = Self::BITS as usize;
            const ZERO: Self = $zero;
            const ONE: Self = $one;
            const MAX: Self = $max;

            #[inline]
            fn count_ones(self) -> u32 {
                Self::count_ones(self)
            }

            #[inline]
            fn leading_zeros(self) -> u32 {
                Self::leading_zeros(self)
            }

            #[inline]
            fn wrapping_shl(self, rhs: u32) -> Self {
                self.wrapping_shl(rhs)
            }

            #[inline]
            fn wrapping_shr(self, rhs: u32) -> Self {
                self.wrapping_shr(rhs)
            }

            #[inline]
            fn clear_high_bits(self, rhs: u32) -> Self {
                self.wrapping_shl(rhs).wrapping_shr(rhs)
            }

            #[inline]
            fn clear_low_bits(self, rhs: u32) -> Self {
                self.wrapping_shr(rhs).wrapping_shl(rhs)
            }
        }
    };
}

impl_bitcontaineritem!(u8, 0u8, 1u8, 0xFFu8);
impl_bitcontaineritem!(u16, 0u16, 1u16, 0xFFFFu16);
impl_bitcontaineritem!(u32, 0u32, 1u32, 0xFFFFFFFFu32);
impl_bitcontaineritem!(u64, 0u64, 1u64, 0xFFFFFFFFFFFFFFFFu64);

// BitContainer is the basic building block for internal storage
// BitVec is expected to be aligned properly
pub trait BitContainer<const L: usize>:
    Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Eq
    + Sized
    + Copy
    + Clone
    + Debug
    + From<[Self::Element; L]>
{
    type Element: BitContainerItem;
    const BIT_WIDTH: usize;
    const ELEMENT_BIT_WIDTH: usize;
    const LANES: usize;
    const ZERO_ELEMENT: Self::Element;
    const ONE_ELEMENT: Self::Element;
    const MAX_ELEMENT: Self::Element;
    const ZERO: Self;
    const MAX: Self;
    fn to_array(self) -> [Self::Element; L];
    fn and_inplace(&mut self, rhs: &Self);
    fn or_inplace(&mut self, rhs: &Self);
    fn xor_inplace(&mut self, rhs: &Self);
}

macro_rules! impl_bitcontainer {
    ($type: ty, $elem_type: ty, $lanes: expr) => {
        impl BitContainer<$lanes> for $type {
            type Element = $elem_type;
            const BIT_WIDTH: usize = ($lanes * <$elem_type>::BIT_WIDTH) as usize;
            const ELEMENT_BIT_WIDTH: usize = <$elem_type>::BIT_WIDTH;
            const LANES: usize = $lanes;
            const ZERO_ELEMENT: $elem_type = <$elem_type>::ZERO;
            const ONE_ELEMENT: $elem_type = <$elem_type>::ONE;
            const MAX_ELEMENT: $elem_type = <$elem_type>::MAX;
            const ZERO: Self = <$type>::splat(0);
            const MAX: Self = <$type>::splat(<$elem_type>::MAX);

            #[inline]
            fn to_array(self) -> [$elem_type; $lanes] {
                <$type>::to_array(self)
            }

            #[inline]
            fn and_inplace(&mut self, rhs: &Self) {
                *self &= rhs;
            }

            #[inline]
            fn or_inplace(&mut self, rhs: &Self) {
                *self |= rhs;
            }

            #[inline]
            fn xor_inplace(&mut self, rhs: &Self) {
                *self ^= rhs;
            }
        }
    };
}

impl_bitcontainer!(u8x16, u8, 16);
impl_bitcontainer!(u16x8, u16, 8);
impl_bitcontainer!(u32x4, u32, 4);
impl_bitcontainer!(u32x8, u32, 8);
impl_bitcontainer!(u64x2, u64, 2);
impl_bitcontainer!(u64x4, u64, 4);
