use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
use std::ops::{Index, IndexMut};
use std::ptr::NonNull;
use std::{alloc, fmt, io, slice};

/// A vector of bytes that aligns its memory to 16 bytes.
pub struct AlignedVec {
    ptr: NonNull<u8>,
    cap: usize,
    len: usize,
}

impl Drop for AlignedVec {
    #[inline]
    fn drop(&mut self) {
        if self.cap != 0 {
            unsafe {
                alloc::dealloc(self.ptr.as_ptr(), self.layout());
            }
        }
    }
}

impl AlignedVec {
    pub const ALIGNMENT: usize = 16;

    #[inline]
    pub fn new() -> Self {
        AlignedVec {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            Self::new()
        } else {
            let ptr = unsafe {
                alloc::alloc(alloc::Layout::from_size_align_unchecked(
                    capacity,
                    Self::ALIGNMENT,
                ))
            };
            Self {
                ptr: NonNull::new(ptr).unwrap(),
                cap: capacity,
                len: 0,
            }
        }
    }

    #[inline]
    fn layout(&self) -> alloc::Layout {
        unsafe { alloc::Layout::from_size_align_unchecked(self.cap, Self::ALIGNMENT) }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    fn change_capacity(&mut self, new_cap: usize) {
        if new_cap != self.cap {
            let new_ptr = unsafe { alloc::realloc(self.ptr.as_ptr(), self.layout(), new_cap) };
            self.ptr = NonNull::new(new_ptr).unwrap();
            self.cap = new_cap;
        }
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        if self.len == 0 {
            self.clear()
        } else {
            self.change_capacity(self.len);
        }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let new_cap = self.len + additional;
        if new_cap > self.cap {
            let new_cap = new_cap
                .checked_next_power_of_two()
                .expect("cannot reserve a larger AlignedVec");
            if self.cap == 0 {
                let new_ptr = unsafe {
                    alloc::alloc(alloc::Layout::from_size_align_unchecked(
                        new_cap,
                        Self::ALIGNMENT,
                    ))
                };
                self.ptr = NonNull::new(new_ptr).unwrap();
                self.cap = new_cap;
            } else {
                let new_ptr = unsafe { alloc::realloc(self.ptr.as_ptr(), self.layout(), new_cap) };
                self.ptr = NonNull::new(new_ptr).unwrap();
                self.cap = new_cap;
            }
        }
    }
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: u8) {
        if new_len > self.len {
            let additional = new_len - self.len;
            self.reserve(additional);
            unsafe {
                std::ptr::write_bytes(self.ptr.as_ptr().add(self.len), value, additional);
            }
        }
        unsafe {
            self.set_len(new_len);
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn extend_from_slice(&mut self, other: &[u8]) {
        if !other.is_empty() {
            self.reserve(other.len());
            unsafe {
                core::ptr::copy_nonoverlapping(
                    other.as_ptr(),
                    self.as_mut_ptr().add(self.len()),
                    other.len(),
                );
            }
            self.len += other.len();
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            None
        } else {
            let result = self[self.len - 1];
            self.len -= 1;
            Some(result)
        }
    }

    #[inline]
    pub fn push(&mut self, value: u8) {
        unsafe {
            self.reserve(1);
            self.as_mut_ptr().add(self.len).write(value);
            self.len += 1;
        }
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        let new_cap = self
            .len
            .checked_add(additional)
            .and_then(|n| n.checked_next_power_of_two())
            .expect("reserve amount overflowed");
        self.change_capacity(new_cap);
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.len = new_len;
    }

    #[inline]
    pub fn into_boxed_slice(self) -> Box<[u8]> {
        self.into_vec().into_boxed_slice()
    }

    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        Vec::from(self.as_ref())
    }
}

impl From<AlignedVec> for Vec<u8> {
    #[inline]
    fn from(aligned: AlignedVec) -> Self {
        aligned.to_vec()
    }
}

impl AsMut<[u8]> for AlignedVec {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl AsRef<[u8]> for AlignedVec {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Borrow<[u8]> for AlignedVec {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl BorrowMut<[u8]> for AlignedVec {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl Clone for AlignedVec {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            let mut result = AlignedVec::with_capacity(self.len);
            result.len = self.len;
            core::ptr::copy_nonoverlapping(self.as_ptr(), result.as_mut_ptr(), self.len);
            result
        }
    }
}

impl fmt::Debug for AlignedVec {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl Default for AlignedVec {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for AlignedVec {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for AlignedVec {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<I: slice::SliceIndex<[u8]>> Index<I> for AlignedVec {
    type Output = <I as slice::SliceIndex<[u8]>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<I: slice::SliceIndex<[u8]>> IndexMut<I> for AlignedVec {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl io::Write for AlignedVec {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        let len = bufs.iter().map(|b| b.len()).sum();
        self.reserve(len);
        for buf in bufs {
            self.extend_from_slice(buf);
        }
        Ok(len)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl PartialEq for AlignedVec {
    fn eq(&self, other: &Self) -> bool {
        self[..] == other[..]
    }
}

// SAFETY: AlignedVec is safe to send to another thread
unsafe impl Send for AlignedVec {}

// SAFETY: AlignedVec is safe to share between threads
unsafe impl Sync for AlignedVec {}

impl Unpin for AlignedVec {}
