use rkyv::AlignedVec;

pub trait Buffer: AsMut<[u8]> {
    fn with_length(length: usize) -> Self;
}

impl Buffer for Vec<u8> {
    fn with_length(length: usize) -> Self {
        vec![0; length]
    }
}

#[cfg(feature = "rkyv")]
impl Buffer for rkyv::AlignedVec {
    fn with_length(length: usize) -> Self {
        let mut v = AlignedVec::with_capacity(length);
        unsafe {
            v.as_mut_ptr().write_bytes(0, length);
            v.set_len(length);
        }
        v
    }
}
