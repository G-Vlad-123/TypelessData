
/*!
TODO
 */

#![cfg_attr(feature = "ptr_metadata", feature(ptr_metadata))]
#![cfg_attr(feature = "allocator_api", feature(allocator_api))]
#![cfg_attr(feature = "allocator_api", feature(try_reserve_kind))]
#![cfg_attr(feature = "new_range_api", feature(new_range_api))]
#![cfg_attr(feature = "generic_const_exprs", feature(generic_const_exprs))]
#![no_std]

#![warn(missing_docs)]
#![warn(missing_abi)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod array;
pub mod slice;
#[cfg(feature = "alloc")]
pub mod boxed;

mod const_ops;
// pub use const_ops::*;

pub mod idx;

#[cfg(feature = "ptr_metadata")]
trait GetSizeOf<T: ?Sized> {
    fn size(&self) -> usize;
}

#[cfg(feature = "ptr_metadata")]
impl<T: Sized> GetSizeOf<T> for () {
    #[inline] fn size(&self) -> usize { core::mem::size_of::<T>() }
}
#[cfg(feature = "ptr_metadata")]
impl<T: ?Sized> GetSizeOf<T> for usize {
    #[inline] fn size(&self) -> usize { *self }
}
#[cfg(feature = "ptr_metadata")]
impl<T: ?Sized> GetSizeOf<T> for core::ptr::DynMetadata<T> {
    #[inline] fn size(&self) -> usize { self.size_of() }
}

#[doc()]
pub struct DocTest;

/**
The main trait of this crate.

This trait represents all and gives all functionality for all data structures.

If you want all the functionality of this trait in a const context consider using [`DataArray`](array::DataArray).
(Note: Some functionality is missing at the moment, but everythign that can be implemented in a const context
shoudl be implemented here.)

If a new method get's added, it will first get added to this trait so it can reach the most general use case.

# SAFETY
To make sure you are not breaking anything you must make sure that all safety guarantees
and requirments are used. All unsafe functions will mention what they want.

This trait also is not made with panics in mind, so unless mentioned otherwise
you should try to not have panics in this trait. As it could lead to corrpt data in surtun positions,
especially if you use it in any context where [`Sync`] and [`Send`] matter.

This trait does not assume how, where and why you store your data.

# Default Implementation Assumptions
- All the functions uses size as if it represented
all the usable data, but it's up to the implementor to decide weatehr or not to
have the size of a data structure represent the amount of usable data in the
current moment (eg: Vec's len) or the capacity of how much data can be stored
(eg: Vec's capacity) or any other mesurment that represents how much data can be stored.
- All indecies strictly smaller then the output given by [`size`](RawDataStructure::size)
are valid indecies.

# Implementation Details
Finnally: if 'idx' is index, identitifaction, id extras, io do externals or anythign else is up to the implementor
(the termenology does not matter as long as you know what the implementor is refering to)
 */
pub unsafe trait RawDataStructure {
    /// Get's the current size of the data structure.
    fn size(&self) -> usize;

    /// Checks weather an index at a surtun location with a surtun size is readable.
    /// 
    /// Whatever this means depeands on the implementation,
    /// the implementor should mention what this means exacly though.
    /// 
    /// If there is no mention though by default you can assume that all
    /// this function checks for is that the slice of size `size` starting from
    /// the index `idx` fits fully within the allocated/stored memory region of
    /// the data structure.
    /// (aka: `idx + size < self.size()`)
    /// 
    /// Meaning of each input:
    /// - `idx`: The starting index of the check.
    /// - `size`: The amount of space required starting from `idx` (in bytes)
    /// 
    /// If this function returns [`Ok(())`](Ok) then it should **always** be
    /// safe to use an unsafe read function that asks for the read data to
    /// not be from outside the data structure as long as all the other
    /// safety requirments (if any) are also satisfied.
    fn read_validity(&self, idx: usize, size: usize) -> Result<(), idx::IdxError>;

    /// Checks weather an index at a surtun location with a surtun size is writable.
    /// 
    /// If [`read_validity`](RawDataStructure::read_validity) gives an
    /// error this is **not** guaranteed to give an error too.
    /// 
    /// The meaning of this function and [`read_validity`](RawDataStructure::read_validity)
    /// are guaranteed to match though.
    /// 
    /// By default this function just calls [`read_validity`](RawDataStructure::read_validity).
    /// 
    /// If this function returns [`Ok(())`](Ok) then it should **always** be
    /// safe to use an unsafe write function that asks for the writing location of the data to
    /// not be from outside the data structure as long as all the other
    /// safety requirments (if any) are also satisfied.
    #[inline]
    fn write_validity(&self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        self.read_validity(idx, size)
    }

    /// Checks weather an index at a surtun location with a surtun size is readable and writable.
    /// 
    /// If eather [`read_validity`](RawDataStructure::read_validity) or [`write_validity`](RawDataStructure::write_validity)
    /// give an error this function is guaranteed to give an error.
    /// 
    /// If both [`read_validity`](RawDataStructure::read_validity) and [`write_validity`](RawDataStructure::write_validity)
    /// return [`Ok(())`](Ok) then this function is guaranteed to also return [`Ok(())`](Ok).
    /// 
    /// The default implementation just calls both functions and returns an error if eather one errors, otherwise [`Ok(())`](Ok).
    /// But for omtimization purpaces you may change this function's implementation, but it works in all cases by default.
    fn full_validity(&self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        self.read_validity(idx, size)?;
        self.write_validity(idx, size)
    }

    /// Writes the given value at the given index.
    /// 
    /// If you want to store a [?Sized](Sized) value use [write_unsized](RawDataStructure::write_unsized)
    /// 
    /// # ERRORS
    /// Will return an error if the write function catches
    /// it'self trying to write in a memory region that is
    /// not assigned to the data structure.
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    unsafe fn write<T: Sized>(&mut self, idx: usize, value: core::mem::ManuallyDrop<T>) -> Result<(), (core::mem::ManuallyDrop<T>, idx::IdxError)> {
        if let Err(err) = self.write_validity(idx, core::mem::size_of::<T>()) {
            return Err((value, err));
        }

        self.write_unchecked(idx, value);
        Ok(())
    }

    /// Writes the given value at the given index.
    /// 
    /// If you want to store a [?Sized](Sized) value use [write_unsized](RawDataStructure::write_unsized)
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure no data is written to a region outside of the specified data structure.
    unsafe fn write_unchecked<T: Sized>(&mut self, idx: usize, value: core::mem::ManuallyDrop<T>) {
        self.write_unsized_unchecked(idx, &value)
    }

    /// Fills with `0`'s the specified bytes
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    unsafe fn write_zeroes(&mut self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        self.write_validity(idx, size)?;
        self.write_zeroes_unchecked(idx, size);
        Ok(())
    }

    /// Fills with `0`'s the specified bytes
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure no data is written to a region outside of the specified data structure.
    unsafe fn write_zeroes_unchecked(&mut self, idx: usize, size: usize);

    /// Fills with `1`'s the specified bytes
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    unsafe fn write_ones(&mut self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        self.write_validity(idx, size)?;
        self.write_ones_unchecked(idx, size);
        Ok(())
    }

    /// Fills with `1`'s the specified bytes
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure no data is written to a region outside of the specified data structure.
    unsafe fn write_ones_unchecked(&mut self, idx: usize, size: usize);

    /// Writes the given value at the given index.
    /// 
    /// This method performs a shallow copy (the)
    /// 
    /// This method takes ownership of T, the reason why
    /// a box is not used is to avoid needless heap alocations.
    /// 
    /// If you want to store a sized value it
    /// is recomended to use [write](RawDataStructure::write) instead.
    /// 
    /// # PANICS
    /// Will panic if a null pointer is given.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure that the value is not used again after being given to this funtion
    /// (eg: using [`mem::forget`](core::mem::forget) or moving the value into a [`ManuallyDrop`](core::mem::ManuallyDrop))
    unsafe fn write_unsized<T: ?Sized>(&mut self, idx: usize, value: *const core::mem::ManuallyDrop<T>) -> Result<(), idx::IdxError> {
        self.write_validity(
            idx,
            core::mem::size_of_val::<core::mem::ManuallyDrop<T>>(
                match value.as_ref() {
                    Some(some) => some,
                    None => unimplemented!(),
                }
            )
        )?;

        self.write_unsized_unchecked(idx, value);

        Ok(())
    }

    /// Writes the given value at the given index.
    /// 
    /// This method performs a shallow copy (the)
    /// 
    /// This method takes ownership of T, the reason why
    /// a box is not used is to avoid needless heap alocations.
    /// 
    /// If you want to store a sized value it
    /// is recomended to use [write](RawDataStructure::write) instead.
    /// 
    /// # PANICS
    /// Will panic if a null pointer is given.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure that the value is not used again after being given to this funtion
    /// (eg: using [`mem::forget`](core::mem::forget) or moving the value into a [`ManuallyDrop`](core::mem::ManuallyDrop))
    /// - Make sure no data is written to a region outside of the specified data structure
    unsafe fn write_unsized_unchecked<T: ?Sized>(&mut self, idx: usize, value: *const core::mem::ManuallyDrop<T>);

    /// Returns a pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to be non-null.
    // Not using NonNull is intentional
    fn read<T: Sized>(&self, idx: usize) -> Result<*const T, crate::idx::IdxError> {
        self.read_validity(idx, core::mem::size_of::<T>())?;

        Ok(
            unsafe {
                self.read_unchecked::<T>(idx)
            }
        )
    }

    /// Returns a refrence to the specified data region.
    /// 
    /// # SAFETY
    /// - Make sure the data is aligned
    /// - Make sure the data is valid
    // Not using NonNull is intentional
    unsafe fn read_ref<T: Sized>(&self, idx: usize) -> Result<&T, crate::idx::IdxError> {
        self.read::<T>(idx).map(
            #[inline] |ptr| unsafe {
                ptr.as_ref() // SAFETY: The caller msut uphold the safety contract.
                   .unwrap_unchecked() // SAFETY: read can never return a null ptr.
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
    unsafe fn read_unchecked<T: Sized>(&self, idx: usize) -> *const T;

    /// Returns a refrence to the specified data region.
    /// 
    /// # SAFETY
    /// - Make sure data isn't read from outside the data structure
    /// - Make sure the data is aligned
    /// - Make sure the data is valid
    unsafe fn read_ref_unchecked<T: Sized>(&self, idx: usize) -> &T {
        unsafe {
            self.read_unchecked::<T>(idx) // SAFETY: The caller must uphold the safety contract.
                .as_ref() // SAFETY: The caller must uphold the safety contract.
                .unwrap_unchecked() // SAFETY: read can never return a null ptr.
        }
    }

    /// Returns a mutable pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    // Not using NonNull is intentional
    fn read_mut<T: Sized>(&mut self, idx: usize) -> Result<*mut T, crate::idx::IdxError> {
        self.read_validity(idx, core::mem::size_of::<T>())?;

        Ok(
            // SAFETY: The data will always be from within the data structure
            unsafe { self.read_mut_unchecked::<T>(idx) }
        )
    }

    /// Returns a mutable pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    /// 
    /// # SAFETY
    /// - Make sure the data is aligned
    /// - Make sure the data is valid
    /// - Make sure there is only one refrence to
    ///   the specified data while whis refrence exists
    unsafe fn read_ref_mut<T: Sized>(&mut self, idx: usize) -> Result<&mut T, crate::idx::IdxError> {
        self.read_mut::<T>(idx).map(
            #[inline] |ptr| unsafe {
                ptr.as_mut() // SAFETY: The caller msut uphold the safety contract.
                   .unwrap_unchecked() // SAFETY: read can never return a null ptr.
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
    unsafe fn read_mut_unchecked<T: Sized>(&mut self, idx: usize) -> *mut T;

    /// Returns a mutable pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    /// 
    /// # SAFETY
    /// - Make sure data isn't read from outside the data structure
    /// - Make sure the data is aligned
    /// - Make sure the data is valid
    /// - Make sure there is only one refrence to the targeted value
    unsafe fn read_ref_mut_unchecked<T: Sized>(&mut self, idx: usize) -> &mut T {
        unsafe {
            self.read_mut_unchecked::<T>(idx) // SAFETY: The caller must uphold the safety contract.
                .as_mut() // SAFETY: The caller msut uphold the safety contract.
                .unwrap_unchecked() // SAFETY: read can never return a null ptr.
        }
    }

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read](RawDataStructure::read) instead.
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    fn read_unsized<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> Result<*const T, idx::IdxError>
    where T::Metadata: crate::GetSizeOf<T> {
        self.read_validity(idx, meta.size())?;
        
        Ok(
            // SAFETY: The data will always be from within the data structure
            unsafe { self.read_unsized_unchecked(idx, meta) }
        )
    }

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_red](RawDataStructure::read_ref) instead.
    /// 
    /// # SAFETY
    /// - Make sure the data is aligned
    /// - Make sure the data is valid
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    unsafe fn read_unsized_ref<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> Result<&T, idx::IdxError>
    where T::Metadata: crate::GetSizeOf<T> {
        self.read_unsized::<T>(idx, meta).map(
            #[inline] |ptr| unsafe {
                ptr.as_ref() // SAFETY: The caller msut uphold the safety contract.
                   .unwrap_unchecked() // SAFETY: read can never return a null ptr.
            }
        )
    }

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_unchecked](RawDataStructure::read_unchecked) instead.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    unsafe fn read_unsized_unchecked<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> *const T;

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_ref_unchecked](RawDataStructure::read_ref_unchecked) instead.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    unsafe fn read_unsized_ref_unchecked<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> &T
    where T::Metadata: crate::GetSizeOf<T> {
        self.read_unsized_unchecked::<T>(idx, meta)
            .as_ref()
            .unwrap_unchecked()
    }

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_mut](RawDataStructure::read_mut) instead.
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    fn read_unsized_mut<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> Result<*mut T, idx::IdxError>
    where T::Metadata: crate::GetSizeOf<T> {
        self.read_validity(idx, meta.size())?;
        
        Ok(
            // SAFETY: The data will always be from within the data structure
            unsafe { self.read_unsized_mut_unchecked(idx, meta) }
        )
    }

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_mut](RawDataStructure::read_mut) instead.
    /// 
    /// # SAFETY
    /// - Make sure the data is aligned
    /// - Make sure the data is valid
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    unsafe fn read_unsized_ref_mut<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> Result<&mut T, idx::IdxError>
    where T::Metadata: crate::GetSizeOf<T> {
        self.read_unsized_mut::<T>(idx, meta).map(
            |ptr| unsafe {
                ptr.as_mut() // SAFETY: The caller must uphold this safety contract
                   .unwrap_unchecked() // SAFETY: the ptr can not be null
            }
        )
    }
    
    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_mut_unchecked](RawDataStructure::read_mut_unchecked) instead.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    unsafe fn read_unsized_mut_unchecked<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> *mut T;

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_mut_unchecked](RawDataStructure::read_mut_unchecked) instead.
    /// 
    /// # SAFETY
    /// - Make sure data isn't read from outside the data structure
    /// - Make sure the data is aligned
    /// - Make sure the data is valid
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    unsafe fn read_unsized_ref_mut_unchecked<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> &mut T {
        self.read_unsized_mut_unchecked::<T>(idx, meta) // SAFETY: Up to the caller to uphold this safety contract
            .as_mut() // SAFETY: Up to the caller to uphold this safety contract
            .unwrap_unchecked() // SAFETY: the ptr can not be null
    }

    /// Takes the value from the specified region.
    /// 
    /// Note: This does NOT zero out the specified region
    /// 
    /// # Safety
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the data gotten from inside is a valid T
    unsafe fn take<T: Sized>(&self, idx: usize) -> Result<T, idx::IdxError> {
        self.write_validity(idx, core::mem::size_of::<T>())?;
        Ok(self.take_unchecked(idx))
    }

    /// Takes the value from the specified region.
    /// 
    /// Note: This does NOT zero out the specified region
    /// 
    /// # Safety
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the data gotten from inside is a valid T.
    /// - Make sure data isn't taken from outside the data structure.
    unsafe fn take_unchecked<T: Sized>(&self, idx: usize) -> T;

    /// Takes the value from the specified region.
    /// 
    /// Note: This does NOT zero out the specified region
    /// 
    /// # Safety
    /// - Make sure the data gotten from inside is a valid T
    unsafe fn take_zeroed<T: Sized>(&mut self, idx: usize) -> Result<T, idx::IdxError> {
        let take: T = self.take(idx)?;
        // SAFETY: If this would have been an invalid operation, self.take() would ahve returned an error.
        self.write_zeroes_unchecked(idx, core::mem::size_of::<T>());
        Ok(take)
    }

    /// Takes the value from the specified region.
    /// 
    /// Note: This does NOT zero out the specified region
    /// 
    /// # Safety
    /// - Make sure the data gotten from inside is a valid T
    /// - Make sure data isn't taken from outside the data structure.
    unsafe fn take_zeroed_unchecked<T: Sized>(&mut self, idx: usize) -> T {
        let take: T = self.take_unchecked(idx);
        self.write_zeroes_unchecked(idx, core::mem::size_of::<T>());
        take
    }

    /// Takes the value from the specified region and writes a new value in it's palce.
    /// 
    /// # Safety
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the data gotten from inside is a valid T
    unsafe fn replace<T: Sized>(&mut self, idx: usize, value: core::mem::ManuallyDrop<T>) -> Result<T, (core::mem::ManuallyDrop<T>, idx::IdxError)> {
        if let Err(err) = self.full_validity(idx, core::mem::size_of::<T>()) {
            return Err((value, err));
        }

        Ok(self.replace_unchecked(idx, value))
    }

    /// Takes the value from the specified region and writes a new value in it's palce.
    /// 
    /// # Safety
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the data gotten from inside is a valid T
    /// - Make sure data isn't taken from outside the data structure.
    unsafe fn replace_unchecked<T: Sized>(&mut self, idx: usize, value: core::mem::ManuallyDrop<T>) -> T {
        let take = self.take_unchecked::<T>(idx);
        self.write_unchecked(idx, value);
        take
    }

    /// Clones the entire chunk of data.
    /// 
    /// # ERRORS
    /// If the sizes of the two data slices do not match, then an error is returned,
    /// where the first usize is the size of `self` and the second is the size of the given data structure.
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    unsafe fn clone_from(&mut self, data: &Self) -> Result<(), (usize, usize)> {
        if self.size() != data.size() {
            return Err((self.size(), data.size()));
        }

        self.clone_from_unchecked(data);

        Ok(())
    }

    /// Clones the entire chunk of data.
    /// 
    /// # PANICS
    /// This might or might not panic if the sizes of the two
    /// data structures do not match, depeanding on the implementor.
    /// But even if it can panic when the sized do not match it's
    /// still could just not.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the sizes of the two data structures match.
    unsafe fn clone_from_unchecked(&mut self, data: &Self);

    /// The smallest chunk of data that is used by this particular data structure.
    type DataByte;

    /// Get's the repr at a single index of the byte.
    /// 
    /// # PANICS
    /// It may or may not panic if the given index is
    /// an invalid one.
    /// 
    /// # SAFETY
    /// - Make sure you can get the byte at the given index.
    unsafe fn get_at_idx(&self, idx: usize) -> Self::DataByte;

    /// Set's the repr at a single index of the byte.
    /// 
    /// # PANICS
    /// It may or may not panic if the given index is
    /// an invalid one.
    /// 
    /// # SAFETY
    /// - Make sure you can set the byte at the given index.
    /// - Make sure to not corrupt any data when setting it by
    /// overwriting the data of a part.
    unsafe fn set_at_idx(&mut self, idx: usize, value: Self::DataByte);
}

/// This trait is ment for slicing the 
pub trait DataStructureSlice: RawDataStructure {
    /// Gets a subslice of the whole data structure.
    /// 
    /// Giving in a full range will always give a slice to the entire data slice
    fn get(&self, idx: impl idx::Idx) -> Option<&slice::DataSlice> {
        if self.size() == 0 { return None }
        
        use core::ops::Bound::*;

        match idx.start() {
            Unbounded => (),
            Included(idx) => if idx >= self.size() { return None },
            Excluded(idx) => if idx.saturating_add(1) >= self.size() { return None },
        };

        match idx.end() {
            Unbounded => (),
            Included(idx) => if idx >= self.size() { return None },
            Excluded(idx) => if idx > self.size() { return None },
        };

        unsafe {
            self.get_unchecked(idx)
                .as_ref()
        }
    }

    /// Gets a mutable subslice of the whole data structure.
    /// 
    /// Giving in a full range will always give a slice to the entire data slice
    fn get_mut(&mut self, idx: impl idx::Idx) -> Option<&mut slice::DataSlice> {
        if self.size() == 0 { return None }
        
        use core::ops::Bound::*;

        match idx.start() {
            Unbounded => (),
            Included(idx) => if idx >= self.size() { return None },
            Excluded(idx) => if idx.saturating_add(1) >= self.size() { return None },
        };

        match idx.end() {
            Unbounded => (),
            Included(idx) => if idx >= self.size() { return None },
            Excluded(idx) => if idx > self.size() { return None },
        };

        unsafe {
            self.get_mut_unchecked(idx)
                .as_mut()
        }
    }

    /// Gets a subslice of the whole data structure without checking bounds.
    /// 
    /// Giving in a full range will always give a slice to the entire data slice
    /// 
    /// # SAFETY
    /// Make sure the data is not gotten from outside the
    /// reserved memory for the data structure.
    unsafe fn get_unchecked(&self, idx: impl idx::Idx) -> *const slice::DataSlice;

    /// Gets a mutable subslice of the whole data structure without checking bounds.
    /// 
    /// Giving in a full range will always give a slice to the entire data slice
    /// 
    /// # SAFETY
    /// Make sure the data is not gotten from outside the
    /// reserved memory for the data structure.
    unsafe fn get_mut_unchecked(&mut self, idx: impl idx::Idx) -> *mut slice::DataSlice;

    /// Gets a [`DataSlice`] reprezenting the entire data structure
    fn as_data_slice(&self) -> &slice::DataSlice {
        unsafe {
            self
                .get_unchecked(..) // SAFETY: This should always return a valid slice tothe full range
                .as_ref() // SAFETY: The return slice should always be a valid slice.
                .unwrap_unchecked() // SAFETY: The returned ptr can never be null.
        }
    }

    /// Gets a mutable [`DataSlice`] reprezenting the entire data structure
    fn as_data_slice_mut(&mut self) -> &mut slice::DataSlice {
        unsafe {
            self
                .get_mut_unchecked(..) // SAFETY: This should always return a valid slice tothe full range
                .as_mut() // SAFETY: The return slice should always be a valid slice.
                .unwrap_unchecked() // SAFETY: The returned ptr can never be null.
        }
    }
}

/// A trait for constructing data structures allocated on the heap.
pub trait DataStructureAllocConstructor: RawDataStructure + Sized {
    
    /// The error returned by the constructors.
    type ConstructorError where Self: Sized;

    /// Cosntructs an empty data structure.
    fn empty() -> Self where Self: Sized;

    /// Cosntructs a data structure without touching the underlying memory.
    /// 
    /// # ERRORS
    /// Will return a [`ConstructorError`](DataStructure::ConstructorError) if the
    /// construction fails (usually by an allocation error
    fn uninit(size: usize) -> Result<Self, Self::ConstructorError> where Self: Sized;

    /// Cosntructs a data structure filling the underlying memory with `0`'s.
    /// 
    /// # ERRORS
    /// Will return a [`ConstructorError`](DataStructure::ConstructorError) if the
    /// construction fails (usually by an allocation error
    #[inline] fn zeroed(size: usize) -> Result<Self, Self::ConstructorError> where Self: Sized {
        Self::filled(size, 0)
    }

    /// Cosntructs a data structure filling the underlying memory with the given byte.
    /// 
    /// # ERRORS
    /// Will return a [`ConstructorError`](DataStructure::ConstructorError) if the
    /// construction fails (usually by an allocation error
    fn filled(size: usize, byte: u8) -> Result<Self, Self::ConstructorError> where Self: Sized;

    /// Cosntructs a data structure from an array data structure.
    /// 
    /// # ERRORS
    /// Will return a [`ConstructorError`](DataStructure::ConstructorError) if the
    /// construction fails (usually by an allocation error).
    fn from_data_array<const SIZE: usize>(array: crate::array::DataArray<SIZE>) -> Result<Self, Self::ConstructorError> where Self: Sized;

    /// Clones the entire chunk of data.
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    unsafe fn clone(&self) -> Result<Self, Self::ConstructorError> where Self: Sized {
        let mut data: Self = Self::uninit(self.size())?;
        data.clone_from_unchecked(self);
        Ok(data)
    }
}

/// A trait for constructing data structures placed on the stack.
pub trait DataStructureFixedConstructor<const SIZE: usize>: RawDataStructure + Sized {
    // TODO: add traits
}

impl<const SIZE: usize, D: DataStructureAllocConstructor> DataStructureFixedConstructor<SIZE> for D {}

/// An marker trait for types that implement most DataStructure traits.
pub trait DataStructure: RawDataStructure + DataStructureAllocConstructor + DataStructureSlice {}
impl<T> DataStructure for T where T: DataStructureAllocConstructor + DataStructureSlice {}

#[cfg(test)]
mod tests;

// TODO:
//   - Remove the need for serde_derive feature
//   - Add Serialize and Deserialize for Data
