
/*!
This module provides the [`DataSlice`] data structure and all it's associated
functions, methods and items.

[`DataSlice`] is made to be usable any context.

Under the surface a [`DataSlice`] is just a slice of bytes with the special designation
that it is a region of memory where each peace of data and it's type is not determined by
the compiler.

[`DataSlice`] is where all the methods are implemented, [`DataArray`](crate::array::DataArray)
re-implements the const methods but they just inline back to calls to
[`DataSlice's`](DataSlice) methods.

All data structures in this crate implement [`Deref`](core::ops::Deref) and
[`DerefMut`](core::ops::Deref) where the target is [`DataSlice`] except for
[`DataSlice`] it'self.

It is also the return type of all indexing operations (beside get_byte)

[`DataSlice`] will probably not implement [`Deref<Target = [u8]>`](core::ops::Deref)
but it might have one day an operation to return the underlying slice or a part
of it. (This is because a `[u8]` represents a slice of bytes and may be used as an
"array or bytes with no knows size at compile time" or some other meaning, meanwhile
[`DataSlice`] will always represent a chunk of memory assigned to holding data with
types unknown by the compiler.)
 */

#[cfg(feature = "allocator_api")]
#[cfg(feature = "alloc")]
use alloc::alloc::Allocator;

use crate::idx;
#[cfg(feature = "ptr_metadata")]
use crate::GetSizeOf;

use core::mem::ManuallyDrop;

#[cfg(feature = "alloc")]
use crate::{
    alloc::{
        // self,
        boxed::Box,
        sync::Arc,
        rc::Rc,
        vec::Vec,
    },
    boxed::DataBoxed,
    array::DataArray,
};

// #[cfg(feature = "std")]
// use crate::std::sync::Mutex;

#[allow(dead_code)]
type Slice<T> = [T];

/// The slice of a typeless chunk of data.
/// 
/// This provides most of the functionality of the crate.
/// 
/// This struct is just a [\[u8\]](Slice<u8>) underneeth the hood.
#[must_use]
#[repr(transparent)]
pub struct DataSlice {
    pub(crate) inner: [u8]
}

impl DataSlice {
    /// Turns a [`&\[u8\]`](Slice<u8>) into a [`&DataSlice`](DataSlice).
    #[inline]
    pub const fn from_slice(slice: &[u8]) -> &DataSlice {
        unsafe {
            // SAFETY: The underlying data is the same for both a slice and Data
            core::mem::transmute(slice)
        }
    }

    /// Turns a [`&mut \[u8\]`](Slice<u8>) into a [`&mut DataSlice`](DataSlice).
    #[inline]
    pub const fn from_slice_mut(slice: &mut [u8]) -> &mut DataSlice {
        unsafe {
            // SAFETY: The underlying data is the same for both a slice and Data
            core::mem::transmute(slice)
        }
    }
    
    /// Turns a [`&\[u8\]`](Slice<u8>) into a [`&DataSlice`](DataSlice).
    #[inline]
    pub const fn from_slice_ptr(slice: *const [u8]) -> *const DataSlice {
        unsafe {
            // SAFETY: The underlying data is the same for both a slice and Data
            core::mem::transmute(slice)
        }
    }

    /// Turns a [`&mut \[u8\]`](Slice<u8>) into a [`&mut DataSlice`](DataSlice).
    #[inline]
    pub const fn from_slice_ptr_mut(slice: *mut [u8]) -> *mut DataSlice {
        unsafe {
            // SAFETY: The underlying data is the same for both a slice and Data
            core::mem::transmute(slice)
        }
    }

    /// Turns a [Box<\[u8\]>] into a [Box<DataSlice>].
    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg(feature = "allocator_api")]
    pub const fn from_boxed_slice<A: Allocator>(boxed: Box<[u8], A>) -> Box<DataSlice, A> {
        use core::mem::ManuallyDrop;

        union Union<A: Allocator> {
            boxed: ManuallyDrop<Box<[u8], A>>,
            output: ManuallyDrop<Box<DataSlice, A>>,
        }

        ManuallyDrop::into_inner(
            unsafe {
                // SAFETY: The underlying data is the same for both a slice and Data
                Union { boxed: ManuallyDrop::new(boxed) }.output
            }
        )
    }

    /// Turns a [Box<\[u8\]>] into a [Box<DataSlice>].
    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg(not(feature = "allocator_api"))]
    pub const fn from_boxed_slice(slice: Box<[u8]>) -> Box<DataSlice> {
        unsafe {
            // SAFETY: The underlying data is the same for both a slice and Data
            core::mem::transmute(slice)
        }
    }

    /// Get's the current size of the data structure.
    #[inline]
    pub const fn size(&self) -> usize {
        self.inner.len()
    }

    /// Writes the given value at the given index.
    /// 
    /// If you want to store a [?Sized](Sized) value use [write_unsized](DataSlice::write_unsized)
    /// 
    /// # ERRORS
    /// Will return an error if the write function catches
    /// it'self trying to write in a memory region that is
    /// not assigned to the data structure.
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    pub const unsafe fn write<T: Sized>(&mut self, idx: usize, value: ManuallyDrop<T>) -> Result<(), (ManuallyDrop<T>, idx::IdxError)> {
        let type_size: usize = core::mem::size_of::<T>();

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

    /// Writes the given value at the given index.
    /// 
    /// If you want to store a [?Sized](Sized) value use [write_unsized](DataSlice::write_unsized)
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure no data is written to a region outside of the specified data structure.
    pub const unsafe fn write_unchecked<T: Sized>(&mut self, idx: usize, value: ManuallyDrop<T>) {
        let ptr: *const u8 = (&value as *const ManuallyDrop<T>).cast();
        let mut at: usize = 0;

        while at < core::mem::size_of::<T>() {
            self.inner[at + idx] = unsafe {
                *ptr.add(at)
            };
            at += 1;
        }

        core::mem::forget(value);
    }

    /// Fills with `0`'s the specified bytes
    /// 
    /// # ERRORS
    /// Will return an error if the write function catches
    /// it'self trying to write in a memory region that is
    /// not assigned to the data structure.
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

    /// Fills with `0`'s the specified bytes
    /// 
    /// # ERRORS
    /// Will return an error if the write function catches
    /// it'self trying to write in a memory region that is
    /// not assigned to the data structure.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure no data is written to a region outside of the specified data structure.
    pub const unsafe fn write_zeroes_unchecked(&mut self, idx: usize, size: usize) {
        let mut at: usize = 0;

        while at < size {
            self.inner[at + idx] = 0x00;
            at += 1;
        }
    }

    /// Fills with `1`'s the specified bytes
    /// 
    /// # ERRORS
    /// Will return an error if the write function catches
    /// it'self trying to write in a memory region that is
    /// not assigned to the data structure.
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

    /// Fills with `1`'s the specified bytes
    /// 
    /// # ERRORS
    /// Will return an error if the write function catches
    /// it'self trying to write in a memory region that is
    /// not assigned to the data structure.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure no data is written to a region outside of the specified data structure.
    pub const unsafe fn write_ones_unchecked(&mut self, idx: usize, size: usize) {
        let mut at: usize = 0;

        while at < size {
            self.inner[at + idx] = 0xFF;
            at += 1;
        }
    }

    /// Writes the given value at the given index.
    /// 
    /// This method performs a shallow copy (the)
    /// 
    /// This method takes ownership of T, the reason why
    /// a box is not used is to avoid needless heap alocations.
    /// 
    /// If you want to store a [Sized] value it
    /// is recomended to use [write](Data::write) instead.
    /// 
    /// # PANICS
    /// Will panic if a null pointer is given.
    /// 
    /// # ERRORS
    /// Will return an error if the write function catches
    /// it'self trying to write in a memory region that is
    /// not assigned to the data structure.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure that the value is not used again after being given to this funtion
    /// (eg: using [`mem::forget`](core::mem::forget) or moving the value into a [ManuallyDrop])
    pub const unsafe fn write_unsized<T: ?Sized>(&mut self, idx: usize, value: *const ManuallyDrop<T>) -> Result<(), idx::IdxError> {
        let type_size: usize = core::mem::size_of_val::<ManuallyDrop<T>>(
            match value.as_ref() {
                Some(some) => some,
                None => unimplemented!(),
            }
        );

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

    /// Writes the given value at the given index.
    /// 
    /// This method performs a shallow copy (the)
    /// 
    /// This method takes ownership of T, the reason why
    /// a box is not used is to avoid needless heap alocations.
    /// 
    /// If you want to store a [Sized] value it
    /// is recomended to use [write](Data::write) instead.
    /// 
    /// # PANICS
    /// Will panic if a null pointer is given.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure that the value is not used again after being given to this funtion
    /// (eg: using [`mem::forget`](core::mem::forget) or moving the value into a [ManuallyDrop])
    /// - Make sure no data is written to a region outside of the specified data structure
    pub const unsafe fn write_unsized_unchecked<T: ?Sized>(&mut self, idx: usize, value: *const ManuallyDrop<T>) {
        let type_size: usize = core::mem::size_of_val::<ManuallyDrop<T>>(
            match value.as_ref() {
                Some(some) => some,
                None => unimplemented!(),
            }
        );
        
        let ptr: *const u8 = value.cast();
        let mut at: usize = 0;

        while at < type_size {
            self.inner[at + idx] = unsafe {
                *ptr.add(at)
            };
            at += 1;
        }
    }

    /// Returns a pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    /// 
    /// This is safe because accesing it'self from a raw pointer is unsafe,
    /// and the user should mark then that the safety of the operation.
    // Not using NonNull is intentional (NonNull is *mut, not *const)
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

    /// Returns a pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    // Not using NonNull is intentional (NonNull is *mut, not *const)
    pub const unsafe fn read_unchecked<T: Sized>(&self, idx: usize) -> *const T {
        unsafe {
            // SAFETY: Must be upheld by the caller.
            (&self.inner as *const [u8]).cast::<T>().add(idx)
        }
    }

    /// Returns a mutable pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    /// 
    /// This is safe because accesing it'self from a raw pointer is unsafe,
    /// and the user should mark then that the safety of the operation.
    // Not using NonNull is intentional (consistancy with read)
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

    /// Returns a mutable pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    // Not using NonNull is intentional (consistancy with read)
    pub const unsafe fn read_mut_unchecked<T: Sized>(&mut self, idx: usize) -> *mut T {
        unsafe {
            // SAFETY: The addr of this ptr + idx is guaranteed to be in
            // the data region given to self.inner, which is guaranteed
            // to be in a valid address by the fact that is exists.
            (&mut self.inner as *mut [u8]).cast::<T>().add(idx)
        }
    }

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read](DataSlice::read) instead.
    /// 
    /// This is safe because accesing it'self from a raw pointer is unsafe,
    /// and the user should mark then that the safety of the operation.
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
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
                unsafe {
                    // SAFETY: The addr of this ptr + idx is guaranteed to be in
                    // the data region given to self.inner, which is guaranteed
                    // to be in a valid address by the fact that is exists.
                    (&self.inner as *const [u8]).cast::<u8>().add(idx)
                },
                meta,
            )
        )
    }

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_unchecked](DataSlice::read_unchecked) instead.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    pub const unsafe fn read_unsized_unchecked<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> *const T
    {
        core::ptr::from_raw_parts(
            unsafe {
                // SAFETY: The safety must be upheld by the caller.
                (&self.inner as *const [u8]).cast::<u8>().add(idx)
            },
            meta,
        )
    }

    /// Returns a mutable pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_mut](DataSlice::read_mut) instead.
    /// 
    /// This is safe because accesing it'self from a raw pointer is unsafe,
    /// and the user should mark then that the safety of the operation.
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
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
                unsafe {
                    // SAFETY: The addr of this ptr + idx is guaranteed to be in
                    // the data region given to self.inner, which is guaranteed
                    // to be in a valid address by the fact that is exists.
                    (&mut self.inner as *mut [u8]).cast::<u8>().add(idx)
                },
                meta,
            )
        )
    }
    
    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_mut_unchecked](DataSlice::read_mut_unchecked) instead.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    pub const unsafe fn read_unsized_mut_unchecked<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> *mut T
    {
        core::ptr::from_raw_parts_mut(
            unsafe {
                // SAFETY: The safety must be upheld by the caller.
                (&mut self.inner as *mut [u8]).cast::<u8>().add(idx)
            },
            meta,
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
    
    /// Takes the value from the specified region.
    /// 
    /// Note: This does NOT zero out the specified region
    /// 
    /// # Safety
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the data gotten from inside is a valid T
    /// - Make sure data isn't read from outside the data structure
    pub const unsafe fn take_unchecked<T: Sized>(&self, idx: usize) -> T {
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

        unsafe {
            // SAFETY?: Safety of a valid value from the copied mem regious is assured by the caller.
            value.assume_init()
        }
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
                // - The safety must be upheld by the caller
                // - The caller's problem here too lol
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

    /// Get's a subslice of the data structure in a const context.
    pub const fn get_const(&self, start: core::ops::Bound<usize>, end: core::ops::Bound<usize>) -> Option<&DataSlice> {
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

    /// Get's a mutable subslice of the data structure in a const context.
    pub const fn get_mut_const(&mut self, start: core::ops::Bound<usize>, end: core::ops::Bound<usize>) -> Option<&mut DataSlice> {
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

    /// Get's a refrence to a subslice of the data structure.
    /// 
    /// If you want to be able to do this in a const context use [get_const](DataSlice::get_const).
    /// 
    /// # Errors
    /// Will return [None] if the given index gets out of bounds.
    #[inline]
    pub fn get(&self, idx: impl idx::Idx) -> Option<&DataSlice> {
        self.get_const(idx.start(), idx.end())
    }

    /// Get's a mutable refrence to a subslice of the data structure.
    /// 
    /// If you want to be able to do this in a const context use [get_mut_const](DataSlice::get_mut_const).
    /// 
    /// # Errors
    /// Will return [None] if the given index gets out of bounds.
    #[inline]
    pub fn get_mut(&mut self, idx: impl idx::Idx) -> Option<&mut DataSlice> {
        self.get_mut_const(idx.start(), idx.end())
    }

    /// Get's the iterator that iterates over the data structure.
    #[inline]
    pub fn iter<'data>(&'data self) -> core::iter::Copied<core::slice::Iter<'data, u8>> {
        self.into_iter()
    }

    /// Get's the iterator that iterates over the data structure with mutable acces.
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
impl From<Box<[u8]>> for Box<DataSlice> {
    #[inline] fn from(boxed: Box<[u8]>) -> Box<DataSlice> {
        DataSlice::from_boxed_slice(boxed)
    }
}

#[cfg(feature = "alloc")]
impl From<Vec<u8>> for Box<DataSlice> {
    #[inline] fn from(vec: Vec<u8>) -> Box<DataSlice> {
        DataSlice::from_boxed_slice(vec.into_boxed_slice())
    }
}

#[cfg(feature = "alloc")]
#[cfg(not(feature = "allocator_api"))]
impl From<DataBoxed> for Box<DataSlice> {
    #[inline] fn from(boxed: DataBoxed) -> Box<DataSlice> {
        DataSlice::from_boxed_slice(boxed.inner)
    }
}

#[cfg(feature = "alloc")]
#[cfg(feature = "allocator_api")]
impl<A: Allocator> From<DataBoxed<A>> for Box<DataSlice, A> {
    #[inline] fn from(boxed: DataBoxed<A>) -> Box<DataSlice, A> {
        DataSlice::from_boxed_slice(boxed.inner)
    }
}

#[cfg(feature = "alloc")]
#[cfg(not(feature = "allocator_api"))]
impl From<DataBoxed> for Arc<DataSlice> {
    #[inline] fn from(boxed: DataBoxed) -> Arc<DataSlice> {
        DataSlice::from_boxed_slice(boxed.inner).into()
    }
}

#[cfg(feature = "alloc")]
#[cfg(feature = "allocator_api")]
impl<A: Allocator> From<DataBoxed<A>> for Arc<DataSlice, A> {
    #[inline] fn from(boxed: DataBoxed<A>) -> Arc<DataSlice, A> {
        DataSlice::from_boxed_slice(boxed.inner).into()
    }
}

// #[cfg(feature = "alloc")]
// #[cfg(not(feature = "allocator_api"))]
// impl From<DataBoxed> for Arc<DataSlice> {
//     #[inline] fn from(boxed: DataBoxed) -> Arc<DataSlice> {
//         DataSlice::from_boxed_slice(boxed.inner).into()
//     }
// }

// #[cfg(feature = "std")]
// #[cfg(not(feature = "allocator_api"))]
// impl From<DataBoxed> for Rc<DataSlice> {
//     #[inline] fn from(boxed: DataBoxed) -> Rc<DataSlice> {
//         DataSlice::from_boxed_slice(boxed.inner).into()
//     }
// }

#[cfg(feature = "alloc")]
#[cfg(feature = "allocator_api")]
impl<A: Allocator> From<DataBoxed<A>> for Rc<DataSlice, A> {
    #[inline] fn from(boxed: DataBoxed<A>) -> Rc<DataSlice, A> {
        DataSlice::from_boxed_slice(boxed.inner).into()
    }
}

// #[cfg(feature = "alloc")]
// #[cfg(not(feature = "allocator_api"))]
// impl From<DataBoxed> for Box<DataSlice> {
//     #[inline] fn from(boxed: DataBoxed) -> Box<DataSlice> {
//         DataSlice::from_boxed_slice(boxed.inner)
//     }
// }

#[cfg(feature = "alloc")]
#[cfg(feature = "allocator_api")]
impl<A: Allocator + Default, const SIZE: usize> From<DataArray<SIZE>> for Box<DataSlice, A> {
    #[inline] fn from(boxed: DataArray<SIZE>) -> Box<DataSlice, A> {
        DataSlice::from_boxed_slice(Box::new_in(boxed.inner, A::default()))
    }
}

#[cfg(feature = "alloc")]
#[cfg(not(feature = "allocator_api"))]
impl<const SIZE: usize> From<DataArray<SIZE>> for Box<DataSlice> {
    #[inline] fn from(boxed: DataArray<SIZE>) -> Box<DataSlice> {
        DataSlice::from_boxed_slice(Box::new(boxed.inner))
    }
}

#[cfg(feature = "alloc")]
#[cfg(feature = "allocator_api")]
impl<A: Allocator + Default, const SIZE: usize> From<DataArray<SIZE>> for Arc<DataSlice, A> {
    #[inline] fn from(boxed: DataArray<SIZE>) -> Arc<DataSlice, A> {
        DataSlice::from_boxed_slice(Box::new_in(boxed.inner, A::default())).into()
    }
}

#[cfg(feature = "alloc")]
#[cfg(not(feature = "allocator_api"))]
impl<const SIZE: usize> From<DataArray<SIZE>> for Arc<DataSlice> {
    #[inline] fn from(boxed: DataArray<SIZE>) -> Arc<DataSlice> {
        DataSlice::from_boxed_slice(Box::new(boxed.inner)).into()
    }
}

#[cfg(feature = "alloc")]
#[cfg(feature = "allocator_api")]
impl<A: Allocator + Default, const SIZE: usize> From<DataArray<SIZE>> for Rc<DataSlice, A> {
    #[inline] fn from(boxed: DataArray<SIZE>) -> Rc<DataSlice, A> {
        DataSlice::from_boxed_slice(Box::new_in(boxed.inner, A::default())).into()
    }
}

#[cfg(feature = "alloc")]
#[cfg(not(feature = "allocator_api"))]
impl<const SIZE: usize> From<DataArray<SIZE>> for Rc<DataSlice> {
    #[inline] fn from(boxed: DataArray<SIZE>) -> Rc<DataSlice> {
        DataSlice::from_boxed_slice(Box::new(boxed.inner)).into()
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

unsafe impl crate::RawDataStructure for DataSlice {
    fn size(&self) -> usize {
        self.size()
    }

    fn read_validity(&self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        if match idx.checked_add(size) {
            Some(size) => size < self.size(),
            None => false,
        } {
            Ok(())
        } else {
            Err(idx::IdxError { idx, data_size: self.size(), type_size: size })
        }
    }

    #[inline]
    fn full_validity(&self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        self.read_validity(idx, size)
    }

    unsafe fn clone_from_unchecked(&mut self, data: &Self) {
        self.inner.copy_from_slice( &data.inner )
    }

    #[inline]
    unsafe fn write_zeroes_unchecked(&mut self, idx: usize, size: usize) {
        self.write_zeroes_unchecked(idx, size)
    }

    #[inline]
    unsafe fn write_ones_unchecked(&mut self, idx: usize, size: usize) {
        self.write_ones_unchecked(idx, size)
    }

    #[inline]
    unsafe fn write_unsized_unchecked<T: ?Sized>(&mut self, idx: usize, value: *const core::mem::ManuallyDrop<T>) {
        self.write_unsized_unchecked(idx, value)
    }

    #[inline]
    unsafe fn read_unchecked<T: Sized>(&self, idx: usize) -> *const T {
        self.read_unchecked(idx)
    }

    #[inline]
    unsafe fn read_mut_unchecked<T: Sized>(&mut self, idx: usize) -> *mut T {
        self.read_mut_unchecked(idx)
    }

    #[inline]
    #[cfg(feature = "ptr_metadata")]
    unsafe fn read_unsized_unchecked<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> *const T {
        self.read_unsized_unchecked(idx, meta)
    }

    #[inline]
    #[cfg(feature = "ptr_metadata")]
    unsafe fn read_unsized_mut_unchecked<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> *mut T {
        self.read_unsized_mut_unchecked(idx, meta)
    }

    #[inline]
    unsafe fn take_unchecked<T: Sized>(&self, idx: usize) -> T {
        self.take_unchecked(idx)
    }

    type DataByte = u8;

    #[inline]
    unsafe fn get_at_idx(&self, idx: usize) -> u8 {
        self.inner[idx]
    }

    #[inline]
    unsafe fn set_at_idx(&mut self, idx: usize, byte: u8) {
        self.inner[idx] = byte;
    }
}

impl crate::DataStructureSlice for DataSlice {
    #[inline]
    unsafe fn get_unchecked(&self, idx: impl idx::Idx) -> *const DataSlice {
        use core::ops::Bound::*;

        // included
        let start: usize = match idx.start() {
            Unbounded => 0,
            Included(idx) => idx,
            Excluded(idx) => idx.saturating_add(1),
        };

        // excluded
        let end: usize = match idx.end() {
            Unbounded => self.size(),
            Included(idx) => idx.saturating_sub(1),
            Excluded(idx) => idx,
        };

        DataSlice::from_slice(
            unsafe {
                core::slice::from_raw_parts(
                    (&self.inner as *const [u8]).cast::<u8>().add(start),
                    end.saturating_sub(start),
                )
            }
        )
    }

    #[inline]
    unsafe fn get_mut_unchecked(&mut self, idx: impl idx::Idx) -> *mut DataSlice {
        use core::ops::Bound::*;

        // included
        let start: usize = match idx.start() {
            Unbounded => 0,
            Included(idx) => idx,
            Excluded(idx) => idx.saturating_add(1),
        };

        // excluded
        let end: usize = match idx.end() {
            Unbounded => self.size(),
            Included(idx) => idx.saturating_sub(1),
            Excluded(idx) => idx,
        };

        DataSlice::from_slice_mut(
            unsafe {
                core::slice::from_raw_parts_mut(
                    (&mut self.inner as *mut [u8]).cast::<u8>().add(start),
                    end.saturating_sub(start),
                )
            }
        )
    }

    #[inline] fn get(&self, idx: impl idx::Idx) -> Option<&DataSlice> { self.get(idx) }
    #[inline] fn get_mut(&mut self, idx: impl idx::Idx) -> Option<&mut DataSlice> { self.get_mut(idx) }

    #[inline] fn as_data_slice(&self) -> &DataSlice { self }
    #[inline] fn as_data_slice_mut(&mut self) -> &mut DataSlice { self }
}
