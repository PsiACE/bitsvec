use crate::*;

#[test]
fn test_bit_vec_count_ones() {
    let mut bitvec = BitVec::ones(1000);
    assert_eq!(bitvec.count_ones(), 1000);
    assert_eq!(bitvec.count_ones_before(500), 500);
    bitvec.set(1500, true);
    assert_eq!(bitvec.count_ones(), 1001);
    assert_eq!(bitvec.count_ones_before(500), 500);
}

#[test]
fn test_bitvec_set_raw_copy() {
    let v = vec![7];
    let buf = v.as_ptr();
    let mut bitvec = unsafe { BitVec::from_raw_copy(buf, 1, 64) };
    let ptr = bitvec.storage.as_mut_ptr();
    let buffer_len = bitvec.storage.len();
    let mut bitvec2 = BitVec::zeros(1);
    unsafe {
        bitvec2.set_raw_copy(ptr, buffer_len, bitvec.nbits);
    }
    assert_eq!(v.len(), 1); // ensure v lives long enough
    assert_eq!(bitvec2.get(0), Some(true));
    assert_eq!(bitvec2.get(1), Some(true));
    assert_eq!(bitvec2.get(2), Some(true));
    assert_eq!(bitvec2.get(3), Some(false));
    assert_eq!(bitvec2.get(63), Some(false));
    assert_eq!(bitvec2.get(64), None);
}

#[test]
fn test_bitvec_set_raw() {
    let mut bitvec = BitVec::ones(1025);
    let ptr = bitvec.storage.as_mut_ptr();
    let buffer_len = bitvec.storage.len();
    let capacity = bitvec.storage.capacity();
    let nbits = bitvec.nbits;
    let spilled = bitvec.storage.spilled();
    let mut bitvec2 = BitVec::zeros(1);
    unsafe {
        std::mem::forget(bitvec);
        assert!(spilled);
        bitvec2.set_raw(ptr, buffer_len, capacity, nbits);
        assert_eq!(bitvec2.get(0), Some(true));
        assert_eq!(bitvec2.get(1024), Some(true));
        assert_eq!(bitvec2.get(1025), None);
    }
}
