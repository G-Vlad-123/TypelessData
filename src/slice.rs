
use crate::idx;
#[cfg(feature = "ptr_metadata")]
use crate::GetSizeOf;

use core::mem::ManuallyDrop;

#[cfg(feature = "alloc")]
use crate::alloc::{
    // self,
    boxed::Box,
    vec::Vec,
};

// #[cfg(feature = "allocator_api")]
// use crate::alloc::{
//     alloc::{
//         Allocator,
//         Global,

//         AllocError,
//     },
// };

#[repr(transparent)]
pub struct DataSlice {
    pub(crate) inner: [u8]
}

impl DataSlice {
    // /// Constructs a new [Data] onto the heap
    // /// 
    // /// The reason this returns an [Option<Box<Data>>] instead of
    // /// a [Box<Data>] directly is to use the [try_new](Box::try_new)
    // /// method when it gets stabelized.
    // /// Currently it is guaranteed to always return [Ok], it's recommended
    // /// to ignroe this as once the allocatir-api stabelizes, the change to use
    // /// the [try_new](Box::try_new) function will not be treated a breaking change.
    // #[cfg(feature = "alloc")]
    // #[inline]
    // pub fn uninit(size: usize) -> Result<Box<Data>, core::convert::Infallible> {
    //     Ok(
    //         unsafe {
    //             // SAFETY: The underlying data is the same for both a slice and Data
    //             core::mem::transmute(Box::<[u8]>::new_uninit_slice(size))
    //         }
    //     )
    // }
    
    // #[cfg(feature = "allocator_api")]
    // #[inline]
    // pub fn uninit_in<A: Allocator>(size: usize, alloc: A) -> Result<Box<Data, A>, AllocError> {
    //     Ok(
    //         unsafe {
    //             // SAFETY: The underlying data is the same for both a slice and Data
    //             core::mem::transmute(Box::<[u8]>::try_new_uninit_slice_in(size, alloc)?)
    //         }
    //     )
    // }

    #[inline]
    pub const fn from_slice(slice: &[u8]) -> &DataSlice {
        unsafe {
            // SAFETY: The underlying data is the same for both a slice and Data
            core::mem::transmute(slice)
        }
    }

    #[inline]
    pub const fn from_slice_mut(slice: &mut [u8]) -> &mut DataSlice {
        unsafe {
            // SAFETY: The underlying data is the same for both a slice and Data
            core::mem::transmute(slice)
        }
    }

    #[inline]
    #[cfg(feature = "alloc")]
    pub const fn from_boxed_slice(slice: Box<[u8]>) -> Box<DataSlice> {
        unsafe {
            // SAFETY: The underlying data is the same for both a slice and Data
            core::mem::transmute(slice)
        }
    }

    #[inline]
    pub const fn size(&self) -> usize {
        self.inner.len()
    }

    /// Writes the given value at the given index.
    /// 
    /// If you want to store a [?Sized](Sized) value use [write_unsized](Data::write_unsized)
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    pub const unsafe fn write<T: Sized>(&mut self, idx: usize, value: ManuallyDrop<T>) -> Result<(), (ManuallyDrop<T>, idx::IdxError)> {
        let type_size: usize = core::mem::size_of_val(&value);

        if match idx.checked_add(type_size) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err((value, idx::IdxError { idx, data_size: self.size(), type_size }))
        }
        
        let ptr: *const u8 = (&value as *const ManuallyDrop<T>).cast();
        let mut at: usize = 0;

        while at < type_size {
            self.inner[at + idx] = unsafe {
                *ptr.add(at)
            };
            at += 1;
        }

        core::mem::forget(value);

        Ok(())
    }

    /// Fills with `0`'s the specified bytes
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    pub const unsafe fn write_zeroes(&mut self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        if match idx.checked_add(size) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: self.size(), type_size: size })
        }
        
        let mut at: usize = 0;

        while at < size {
            self.inner[at + idx] = 0x00;
            at += 1;
        }
        
        Ok(())
    }

    /// Fills with `1`'s the specified bytes
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    pub const unsafe fn write_ones(&mut self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        if match idx.checked_add(size) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: self.size(), type_size: size })
        }
        
        let mut at: usize = 0;

        while at < size {
            self.inner[at + idx] = 0xFF;
            at += 1;
        }
        
        Ok(())
    }

    /// Writes the given value at the given index.
    /// 
    /// This method performs a shallow copy (the)
    /// 
    /// This method takes ownership of T, the reason why
    /// a box is not used is to avoid needless heap alocations.
    /// 
    /// If you want to store a [Sized](Sized) value it
    /// is recomended to use [write](Data::write) instead.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure that the value is not used again after being given to this funtion
    /// (eg: using [`mem::forget`](core::mem::forget) or moving the value into a [ManuallyDrop])
    pub const unsafe fn write_unsized<T: ?Sized>(&mut self, idx: usize, value: *const ManuallyDrop<T>) -> Result<(), idx::IdxError> {
        let type_size: usize = core::mem::size_of_val(&value);

        if match idx.checked_add(type_size) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: self.size(), type_size })
        }
        
        let ptr: *const u8 = value.cast();
        let mut at: usize = 0;

        while at < type_size {
            self.inner[at + idx] = unsafe {
                *ptr.add(at)
            };
            at += 1;
        }

        Ok(())
    }

    /// Returns a pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    // Not using NonNull is intentional
    pub const fn read<T: Sized>(&self, idx: usize) -> Result<*const T, idx::IdxError> {
        if match idx.checked_add(core::mem::size_of::<T>()) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: self.size(), type_size: core::mem::size_of::<T>() })
        }

        Ok(
            unsafe {
                // SAFETY: The addr of this ptr + idx is guaranteed to be in
                // the data region given to self.inner, which is guaranteed
                // to be in a valid address by the fact that is exists.
                (&self.inner as *const [u8]).cast::<T>().add(idx)
            }
        )
    }

    /// Returns a mutable pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    // Not using NonNull is intentional
    pub const fn read_mut<T: Sized>(&mut self, idx: usize) -> Result<*mut T, idx::IdxError> {
        if match idx.checked_add(core::mem::size_of::<T>()) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: self.size(), type_size: core::mem::size_of::<T>() })
        }

        Ok(
            unsafe {
                // SAFETY: The addr of this ptr + idx is guaranteed to be in
                // the data region given to self.inner, which is guaranteed
                // to be in a valid address by the fact that is exists.
                (&mut self.inner as *mut [u8]).cast::<T>().add(idx)
            }
        )
    }

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use (read)[Data::read] instead.
    #[cfg(feature = "ptr_metadata")]
    pub fn read_unsized<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> Result<*const T, idx::IdxError>
    where T::Metadata: crate::GetSizeOf<T>
    {
        if match idx.checked_add(meta.size()) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: self.size(), type_size: meta.size() })
        }

        Ok(
            core::ptr::from_raw_parts(
                (&self.inner as *const [u8]).cast::<u8>(),
                meta,
            )
        )
    }

    /// Returns a mutable pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use (read_mut)[Data::read_mut] instead.
    #[cfg(feature = "ptr_metadata")]
    pub fn read_unsized_mut<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> Result<*mut T, idx::IdxError>
    where T::Metadata: crate::GetSizeOf<T>
    {
        if match idx.checked_add(meta.size()) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: self.size(), type_size: meta.size() })
        }

        Ok(
            core::ptr::from_raw_parts_mut(
                (&mut self.inner as *mut [u8]).cast::<u8>(),
                meta,
            )
        )
    }

    /// Takes the value from the specified region.
    /// 
    /// Note: This does NOT zero out the specified region
    /// 
    /// # Safety
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the data gotten from inside is a valid T
    pub const unsafe fn take<T: Sized>(&self, idx: usize) -> Result<T, idx::IdxError> {
        if match idx.checked_add(core::mem::size_of::<T>()) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: self.size(), type_size: core::mem::size_of::<T>() })
        }

        use core::mem::MaybeUninit;

        let mut value: MaybeUninit<T> = MaybeUninit::uninit();

        let ptr: *mut u8 = core::ptr::from_mut(&mut value).cast();
        
        // SAFETY: Because we copy size_of<T> bytes, this means the copy will be a perfect fit
        ptr.copy_from_nonoverlapping(
            unsafe {
                // SAFETY: The addr of this ptr + idx is guaranteed to be in
                // the data region given to self.inner, which is guaranteed
                // to be in a valid address by the fact that is exists.
                (&self.inner as *const [u8]).cast::<u8>().add(idx)
            },
            core::mem::size_of::<T>(),
        );

        Ok(
            unsafe {
                // SAFETY?: Safety of a valid value from the copied mem regious is assured by the caller.
                value.assume_init()
            }
        )
    }

    /// Takes the value from the specified region and writes a new value in it's palce.
    /// 
    /// # Safety
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the data gotten from inside is a valid T
    pub const unsafe fn replace<T: Sized>(&mut self, idx: usize, value: ManuallyDrop<T>) -> Result<T, idx::IdxError> {
        if match idx.checked_add(core::mem::size_of::<T>()) {
            Some(size) => size >= self.size(),
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: self.size(), type_size: core::mem::size_of::<T>() })
        }
        
        Ok(
            unsafe {
                // SAFETY?:
                // - Due to the ptr always pointing to a valid
                //   memory region and it being non-null,
                //   it will be valid.
                // - ...
                // - The caller's problem lol
                core::ptr::replace(
                    // SAFETY: The addr of this ptr + idx is guaranteed to be in
                    // the data region given to self.inner, which is guaranteed
                    // to be in a valid address by the fact that is exists.
                    (&mut self.inner as *mut [u8]).cast::<T>().add(idx),
                    ManuallyDrop::into_inner(value)
                )
            }
        )
    }

    pub const fn idx_const(&self, start: core::ops::Bound<usize>, end: core::ops::Bound<usize>) -> Option<&DataSlice> {
        if self.size() == 0 { return None }
        
        use core::ops::Bound::*;

        // included
        let start: usize = match start {
            Unbounded => 0,
            Included(idx) => if idx < self.size() { idx } else { return None },
            Excluded(idx) => if idx.saturating_add(1) < self.size() { idx + 1 } else { return None },
        };

        // excluded
        let end: usize = match end {
            Unbounded => self.size(),
            Included(idx) => if idx < self.size() { idx.saturating_sub(1) } else { return None },
            Excluded(idx) => if idx <= self.size() { idx } else { return None },
        };

        Some (
            DataSlice::from_slice(
                unsafe {
                    core::slice::from_raw_parts(
                        (&self.inner as *const [u8]).cast::<u8>().add(start),
                        end.saturating_sub(start),
                    )
                }
            )
        )
    }

    pub const fn idx_mut_const(&mut self, start: core::ops::Bound<usize>, end: core::ops::Bound<usize>) -> Option<&mut DataSlice> {
        if self.size() == 0 { return None }
        
        use core::ops::Bound::*;

        // included
        let start: usize = match start {
            Unbounded => 0,
            Included(idx) => if idx < self.size() { idx } else { return None },
            Excluded(idx) => if idx.saturating_add(1) < self.size() { idx + 1 } else { return None },
        };

        // excluded
        let end: usize = match end {
            Unbounded => self.size(),
            Included(idx) => if idx < self.size() { idx.saturating_sub(1) } else { return None },
            Excluded(idx) => if idx <= self.size() { idx } else { return None },
        };

        Some (
            DataSlice::from_slice_mut(
                unsafe {
                    core::slice::from_raw_parts_mut(
                        (&mut self.inner as *mut [u8]).cast::<u8>().add(start),
                        end.saturating_sub(start),
                    )
                }
            )
        )
    }

    #[inline]
    pub fn idx(&self, idx: impl idx::Idx) -> Option<&DataSlice> {
        self.idx_const(idx.start(), idx.end())
    }

    #[inline]
    pub fn idx_mut(&mut self, idx: impl idx::Idx) -> Option<&mut DataSlice> {
        self.idx_mut_const(idx.start(), idx.end())
    }

    #[inline]
    pub fn iter<'data>(&'data self) -> core::iter::Copied<core::slice::Iter<'data, u8>> {
        self.into_iter()
    }

    #[inline]
    pub fn iter_mut<'data>(&'data mut self) -> core::slice::IterMut<'data, u8> {
        self.into_iter()
    }
}

impl Default for &DataSlice {
    #[inline] fn default() -> Self {
        DataSlice::from_slice(&[])
    }
}

impl Default for &mut DataSlice {
    #[inline] fn default() -> Self {
        DataSlice::from_slice_mut(&mut [])
    }
}

#[cfg(feature = "alloc")]
impl Default for Box<DataSlice> {
    #[inline] fn default() -> Self {
        DataSlice::from_boxed_slice(Box::default())
    }
}

impl<'r> From<&'r [u8]> for &'r DataSlice {
    #[inline] fn from(slice: &'r [u8]) -> &'r DataSlice {
        DataSlice::from_slice(slice)
    }
}

impl<'r> From<&'r mut [u8]> for &'r mut DataSlice {
    #[inline] fn from(slice: &'r mut [u8]) -> &'r mut DataSlice {
        DataSlice::from_slice_mut(slice)
    }
}

#[cfg(feature = "alloc")]
impl<'r> From<Box<[u8]>> for Box<DataSlice> {
    #[inline] fn from(boxed: Box<[u8]>) -> Box<DataSlice> {
        DataSlice::from_boxed_slice(boxed)
    }
}

#[cfg(feature = "alloc")]
impl<'r> From<Vec<u8>> for Box<DataSlice> {
    #[inline] fn from(vec: Vec<u8>) -> Box<DataSlice> {
        DataSlice::from_boxed_slice(vec.into_boxed_slice())
    }
}

impl core::fmt::Debug for DataSlice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;

        match f.width() {
            None | Some(0) => {
                let mut iter = self.inner.iter().copied();
                
                if let Some(byte) = iter.next() {
                    write!(f, "{:02X}", byte)?;
                }
                
                for byte in iter {
                    write!(f, " {:02X}", byte)?;
                }
            }
            Some(width) => {
                let mut iter = self.inner.chunks(width).map(|window| window.iter().copied());
                
                let mut last = match iter.next_back() {
                    Some(last) => last,
                    None => return Ok(()),
                };

                if let Some(mut first) = iter.next() {
                    write!(f, "{:02X}", unsafe {
                        // SAFETY: width can not be 0
                        first.next().unwrap_unchecked()
                    })?;

                    for byte in first {
                        write!(f, " {:02X}", byte)?;
                    }
                }

                for mut chunk in iter {
                    write!(f, "\n{:02X}", unsafe {
                        // SAFETY: width can not be 0
                        chunk.next().unwrap_unchecked()
                    })?;
                    
                    for byte in chunk {
                        write!(f, " {:02X}", byte)?;
                    }
                }

                use core::fmt::Alignment::*;
                
                let padding: usize = match f.align().unwrap_or(Left) {
                    Left => 0,
                    Center => (width - self.size() % width) / 2,
                    Right => width - self.size() % width,
                };

                f.write_char('\n')?;

                for _ in 0..padding {
                    f.write_str("   ")?;
                }

                write!(f, "{:02X}", unsafe {
                    // SAFETY: last would not be given if it had 0 elements
                    last.next().unwrap_unchecked()
                })?;
                
                for byte in last {
                    write!(f, " {:02X}", byte)?;
                }
            },
        }

        Ok(())
    }
}


impl<'data> IntoIterator for &'data DataSlice {
    type Item = u8;
    type IntoIter = core::iter::Copied<core::slice::Iter<'data, u8>>;

    #[inline] fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().copied()
    }
}

impl<'data> IntoIterator for &'data mut DataSlice {
    type Item = &'data mut u8;
    type IntoIter = core::slice::IterMut<'data, u8>;

    #[inline] fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}