
/*!
This module provides the [`DataBoxed`] data structure and all it's associated
functions, methods and items.

[`DataBoxed`] is made to be used when you do not know the size of the data structure
or when you know you want to cahnge the size of the data structure and you wanna save
on memory.

Under the surface a [`DataBoxed`] is just a boxed array, that get's reallocated
when the user needs to change it's size.

[`DataBoxed`] is the only data structure that can not be used in a const context
at all, for now if you want a data strcture in a const cotnext use [`DataArray`](crate::array::DataArray)
and/or [`DataSlice`](crate::slice::DataSlice).

## Note
For now [`DataBoxed`] does not give any support for reallocation, this will be added in a future version.
 */

#[cfg(feature = "allocator_api")]
use alloc::{
    alloc::{
        Allocator,
        Global,

        Layout,

        AllocError,
    },
    collections::TryReserveErrorKind,
};

#[allow(unused_imports)]
use crate::alloc::{
    self,
    boxed::Box,
    collections::TryReserveError,
};
use crate::slice::DataSlice;

/// A boxed typeless chunk of data.
/// 
/// In case you don't know how large a chunk of data you want to have,
/// and to change it's size when it is needed.
/// 
/// This struct was NOT made for frequent reallocations,
/// and is optimized for memory usage.
/// 
/// This struct is just a `Box<\[u8\]>` underneeth the hood.
#[must_use]
#[cfg(feature = "allocator_api")]
// #[optimize(size)]
pub struct DataBoxed<A: Allocator = Global> {
    pub(crate) inner: Box<[u8], A>
}

/// A boxed typeless chunk of data.
/// 
/// In case you don't know how large a chunk of data you want to have,
/// and to change it's size when it is needed.
/// 
/// This struct was NOT made for frequent reallocations,
/// and is optimized for memory usage.
/// 
/// This struct is just a `Box<\[u8\]>` underneeth the hood.
#[must_use]
#[cfg(not(feature = "allocator_api"))]
// #[optimize(size)]
pub struct DataBoxed {
    pub(crate) inner: Box<[u8]>
}

impl DataBoxed {
    /// Initializes a new [DataBoxed] without allocating any data.
    #[inline]
    pub fn empty() -> DataBoxed {
        DataBoxed { inner: Box::new([]) }
    }

    /// Constructs a new [DataBoxed] structure without touching the underling data.
    /// 
    /// Depeanding on if you have the `allocator_api` feature this will:
    /// - (no) Panic if an allocation fails, never returns an error.
    /// - (yes) Returns an error if the allocation fails.
    /// [TryReserveError] is used instead of [AllocError] because the former
    /// is stable and can be cosntructed from an [AllocError] (in the current version)
    /// 
    /// Once `allocator_api` get's stabliized it will jsut always return an error.
    /// 
    /// This method is safe because reading in it'self from the data structure is
    /// an unsafe operation, this function marking that the underlying data does
    /// not matter at all when it starts.
    #[inline]
    pub fn uninit(size: usize) -> Result<DataBoxed, TryReserveError> {
        #[cfg(feature = "allocator_api")]
        return DataBoxed::uninit_in(size, Global);

        #[cfg(not(feature = "allocator_api"))]
        return Ok(
            DataBoxed {
                // SAFETY: The data is ment to be uninitialized.
                inner: unsafe { Box::new_uninit_slice(size).assume_init() }
            }
        );
    }

    #[inline]
    /// Constructs a new [DataArray] structure filled with `0`'s.
    /// 
    /// Depeanding on if you have the `allocator_api` feature this will:
    /// - (no) Panic if an allocation fails, never returns an error.
    /// - (yes) Returns an error if the allocation fails.
    /// [TryReserveError] is used instead of [AllocError] because the former
    /// is stable and can be cosntructed from an [AllocError] (in the current version)
    /// 
    /// Once `allocator_api` get's stabliized it will jsut always return an error.
    pub fn zeroed(size: usize) -> Result<DataBoxed, TryReserveError> {
        #[cfg(feature = "allocator_api")]
        return DataBoxed::zeroed_in(size, Global);

        #[cfg(not(feature = "allocator_api"))]
        DataBoxed::filled(size, 0)
    }

    /// Constructs a new [DataArray] structure filled with whatever byte you give.
    /// 
    /// Depeanding on if you have the `allocator_api` feature this will:
    /// - (no) Panic if an allocation fails, never returns an error.
    /// - (yes) Returns an error if the allocation fails.
    /// [TryReserveError] is used instead of [AllocError] because the former
    /// is stable and can be cosntructed from an [AllocError] (in the current version)
    /// 
    /// Once `allocator_api` get's stabliized it will jsut always return an error.
    pub fn filled(size: usize, byte: u8) -> Result<DataBoxed, TryReserveError> {
        let mut data = DataBoxed::uninit(size)?;
        data.inner.fill(byte);
        Ok(data)
    }
}

#[cfg(feature = "allocator_api")]
impl<A: Allocator> DataBoxed<A> {
    /// Initializes a new [DataBoxed] without allocating any data.
    #[inline]
    pub fn empty_in(alloc: A) -> DataBoxed<A> {
        DataBoxed { inner: Box::new_in([], alloc) }
    }

    /// Constructs a new [DataBoxed] structure without touching the underling data.
    /// 
    /// This method is safe because reading in it'self from the data structure is
    /// an unsafe operation, this function marking that the underlying data does
    /// not matter at all when it starts.
    /// 
    /// Returns an error if the allocation fails.
    /// [TryReserveError] is used instead of [AllocError] because the former
    /// is stable and can be cosntructed from an [AllocError] (in the current version)
    pub fn uninit_in(size: usize, alloc: A) -> Result<DataBoxed<A>, TryReserveError> {
        match Box::try_new_uninit_slice_in(size, alloc) {
            Ok(data) => Ok(
                DataBoxed {
                    // SAFETY: The data is ment to be uninitialized.
                    inner: unsafe { data.assume_init() }
                }
            ),
            Err(AllocError) => Err(
                match Layout::array::<u8>(size) {
                    Ok(layout) => TryReserveErrorKind::AllocError {
                        layout, non_exhaustive: (),
                    },
                    Err(err) => err.into(),
                }.into()
            )
        }
    }

    /// Constructs a new [DataBoxed] structure filled with `0`'s.
    /// 
    /// Returns an error if the allocation fails.
    /// [TryReserveError] is used instead of [AllocError] because the former
    /// is stable and can be cosntructed from an [AllocError] (in the current version)
    pub fn zeroed_in(size: usize, alloc: A) -> Result<DataBoxed<A>, TryReserveError> {
        match Box::try_new_zeroed_slice_in(size, alloc) {
            Ok(data) => Ok(
                DataBoxed {
                    // SAFETY: The data is ment to be zeroed.
                    inner: unsafe { data.assume_init() }
                }
            ),
            Err(AllocError) => Err(
                match Layout::array::<u8>(size) {
                    Ok(layout) => TryReserveErrorKind::AllocError {
                        layout, non_exhaustive: (),
                    },
                    Err(err) => err.into(),
                }.into()
            )
        }
    }

    /// Constructs a new [DataBoxed] structure filled with whatever byte you give.
    /// 
    /// Returns an error if the allocation fails.
    /// [TryReserveError] is used instead of [AllocError] because the former
    /// is stable and can be cosntructed from an [AllocError] (in the current version)
    pub fn filled_in(size: usize, byte: u8, alloc: A) -> Result<DataBoxed<A>, TryReserveError> {
        let mut data = DataBoxed::uninit_in(size, alloc)?;
        data.inner.fill(byte);
        Ok(data)
    }

    /// Get's the allocator of the data structure.
    #[inline]
    pub fn allocator(&self) -> &A {
        Box::allocator(&self.inner)
    }
}

macro_rules! impl_data_boxed {
    (
        $(
            $( $attr:meta )*
            $func:item
        )*
    ) => {
        #[cfg(feature = "allocator_api")]
        impl<A: Allocator> DataBoxed<A> {
            $(
                $( $attr )*
                $func
            )*
        }

        #[cfg(not(feature = "allocator_api"))]
        impl DataBoxed {
            $(
                $( $attr )*
                $func
            )*
        }
    };
}

impl_data_boxed!{
    #[inline]
    /// Get's the current size of the data structure.
    pub const fn size(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(feature = "allocator_api")]
impl<A: Allocator> DataBoxed<A> {
    /// Clones the entire chunk of data.
    /// 
    /// Depeanding on if you have the `allocator_api` feature this will:
    /// - (no) Panic if an allocation fails, never returns an error.
    /// - (yes) Returns an error if the allocation fails.
    /// [TryReserveError] is used instead of [AllocError] because the former
    /// is stable and can be cosntructed from an [AllocError] (in the current version)
    /// 
    /// Once `allocator_api` get's stabliized it will jsut always return an error.
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    pub unsafe fn clone(&self) -> Result<DataBoxed<A>, TryReserveError>
    where A: Clone
    {
        let mut data = DataBoxed::uninit_in(self.size(), self.allocator().clone())?;
        let mut idx: usize = 0;
        
        while idx < self.size() {
            data.inner[idx] = self.inner[idx];
            idx += 1;
        }

        Ok(data)
    }
}

#[cfg(not(feature = "allocator_api"))]
impl DataBoxed {
    /// Clones the entire chunk of data.
    /// 
    /// Depeanding on if you have the `allocator_api` feature this will:
    /// - (no) Panic if an allocation fails, never returns an error.
    /// - (yes) Returns an error if the allocation fails.
    /// [TryReserveError] is used instead of [AllocError] because the former
    /// is stable and can be cosntructed from an [AllocError] (in the current version)
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    pub unsafe fn clone(&self) -> Result<DataBoxed, TryReserveError> {
        let mut data = DataBoxed::uninit(self.size())?;
        let mut idx: usize = 0;
        
        while idx < self.size() {
            data.inner[idx] = self.inner[idx];
            idx += 1;
        }

        Ok(data)
    }
}

#[cfg(feature = "allocator_api")]
impl<A: Allocator> core::ops::Deref for DataBoxed<A> {
    type Target = crate::slice::DataSlice;

    #[inline] fn deref(&self) -> &Self::Target {
        crate::slice::DataSlice::from_slice(&self.inner)
    }
}

#[cfg(not(feature = "allocator_api"))]
impl core::ops::Deref for DataBoxed {
    type Target = crate::slice::DataSlice;

    #[inline] fn deref(&self) -> &Self::Target {
        crate::slice::DataSlice::from_slice(&self.inner)
    }
}

#[cfg(feature = "allocator_api")]
impl<A: Allocator> core::ops::DerefMut for DataBoxed<A> {
    #[inline] fn deref_mut(&mut self) -> &mut Self::Target {
        crate::slice::DataSlice::from_slice_mut(&mut self.inner)
    }
}

#[cfg(not(feature = "allocator_api"))]
impl core::ops::DerefMut for DataBoxed {
    #[inline] fn deref_mut(&mut self) -> &mut Self::Target {
        crate::slice::DataSlice::from_slice_mut(&mut self.inner)
    }
}

#[cfg(feature = "allocator_api")]
impl<A: Allocator> core::fmt::Debug for DataBoxed<A> {
    #[inline] fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <crate::slice::DataSlice as core::fmt::Debug>::fmt(&self, f)
    }
}

#[cfg(not(feature = "allocator_api"))]
impl core::fmt::Debug for DataBoxed {
    #[inline] fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <crate::slice::DataSlice as core::fmt::Debug>::fmt(&self, f)
    }
}

#[cfg(feature = "allocator_api")]
impl<A: Allocator + Default> Default for DataBoxed<A> {
    #[inline] fn default() -> Self { DataBoxed::empty_in(A::default()) }
}

#[cfg(not(feature = "allocator_api"))]
impl Default for DataBoxed {
    #[inline] fn default() -> Self { DataBoxed::empty() }
}

#[cfg(feature = "std")]
impl<'mutex> DerefDataSlice for crate::std::sync::MutexGuard<'mutex, crate::slice::DataSlice> {}

trait DerefDataSlice: core::ops::DerefMut<Target = crate::slice::DataSlice> {}

#[cfg(feature = "allocator_api")]
mod alloc_api_impl {
    use super::*;

    // use alloc::{
    //     sync::Arc,
    //     rc::Rc,
    // };

    use crate::slice::DataSlice;

    impl<A: Allocator> DerefDataSlice for DataBoxed<A> {}
    impl<A: Allocator> DerefDataSlice for Box<DataSlice, A> {}
}

#[cfg(not(feature = "allocator_api"))]
mod alloc_api_impl {
    use super::*;

    use crate::slice::DataSlice;

    impl DerefDataSlice for DataBoxed {}
    impl DerefDataSlice for Box<DataSlice> {}
}

unsafe impl<D: DerefDataSlice> crate::RawDataStructure for D {
    #[inline]
    fn size(&self) -> usize {
        <crate::slice::DataSlice as crate::RawDataStructure>::size(self)
    }

    #[inline]
    fn read_validity(&self, idx: usize, size: usize) -> Result<(), crate::idx::IdxError> {
        <crate::slice::DataSlice as crate::RawDataStructure>::read_validity(self, idx, size)
    }

    #[inline]
    unsafe fn write_zeroes_unchecked(&mut self, idx: usize, size: usize) {
        <crate::slice::DataSlice as crate::RawDataStructure>::write_zeroes_unchecked(self, idx, size)
    }

    #[inline]
    unsafe fn write_ones_unchecked(&mut self, idx: usize, size: usize) {
        <crate::slice::DataSlice as crate::RawDataStructure>::write_ones_unchecked(self, idx, size)
    }

    #[inline]
    unsafe fn write_unsized_unchecked<T: ?Sized>(&mut self, idx: usize, value: *const core::mem::ManuallyDrop<T>) {
        <crate::slice::DataSlice as crate::RawDataStructure>::write_unsized_unchecked(self, idx, value)
    }

    #[inline]
    unsafe fn read_unchecked<T: Sized>(&self, idx: usize) -> *const T {
        <crate::slice::DataSlice as crate::RawDataStructure>::read_unchecked(self, idx)
    }

    #[inline]
    unsafe fn read_mut_unchecked<T: Sized>(&mut self, idx: usize) -> *mut T {
        <crate::slice::DataSlice as crate::RawDataStructure>::read_mut_unchecked(self, idx)
    }

    #[inline]
    unsafe fn read_unsized_unchecked<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> *const T {
        <crate::slice::DataSlice as crate::RawDataStructure>::read_unsized_unchecked(self, idx, meta)
    }

    #[inline]
    unsafe fn read_unsized_mut_unchecked<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> *mut T {
        <crate::slice::DataSlice as crate::RawDataStructure>::read_unsized_mut_unchecked(self, idx, meta)
    }

    #[inline]
    unsafe fn take_unchecked<T: Sized>(&self, idx: usize) -> T {
        <crate::slice::DataSlice as crate::RawDataStructure>::take_unchecked(self, idx)
    }

    #[inline]
    unsafe fn clone_from_unchecked(&mut self, data: &Self) {
        <crate::slice::DataSlice as crate::RawDataStructure>::clone_from_unchecked(self, data)
    }

    type DataByte = u8;

    #[inline]
    unsafe fn get_at_idx(&self, idx: usize) -> Self::DataByte {
        <crate::slice::DataSlice as crate::RawDataStructure>::get_at_idx(self, idx)
    }

    #[inline]
    unsafe fn set_at_idx(&mut self, idx: usize, byte: Self::DataByte) {
        <crate::slice::DataSlice as crate::RawDataStructure>::set_at_idx(self, idx, byte)
    }
}

impl<D: DerefDataSlice> crate::DataStructureSlice for D {
    #[inline]
    unsafe fn get_unchecked(&self, idx: impl crate::idx::Idx) -> *const crate::slice::DataSlice {
        self.deref().get_unchecked(idx)
    }

    #[inline]
    unsafe fn get_mut_unchecked(&mut self, idx: impl crate::idx::Idx) -> *mut crate::slice::DataSlice {
        self.deref_mut().get_mut_unchecked(idx)
    }
}

impl crate::DataStructureAllocConstructor for DataBoxed {
    type ConstructorError = TryReserveError where Self: Sized;

    #[inline]
    fn empty() -> Self where Self: Sized {
        DataBoxed { inner: Box::new([]) }
    }

    #[inline]
    fn uninit(size: usize) -> Result<Self, Self::ConstructorError> where Self: Sized {
        DataBoxed::uninit(size)
    }

    #[inline]
    fn zeroed(size: usize) -> Result<Self, Self::ConstructorError> where Self: Sized {
        DataBoxed::zeroed(size)
    }

    #[inline]
    fn filled(size: usize, byte: u8) -> Result<Self, Self::ConstructorError> where Self: Sized {
        DataBoxed::filled(size, byte)
    }

    #[inline]
    fn from_data_array<const SIZE: usize>(array: crate::array::DataArray<SIZE>) -> Result<Self, Self::ConstructorError> where Self: Sized {
        let mut data = DataBoxed::uninit(SIZE)?;
        data.inner.copy_from_slice(&array.inner);
        Ok(data)
    }

    #[inline]
    unsafe fn clone(&self) -> Result<Self, Self::ConstructorError> where Self: Sized {
        self.clone()
    }
}

impl crate::DataStructureAllocConstructor for Box<crate::slice::DataSlice> {
    type ConstructorError = TryReserveError where Self: Sized;

    #[inline]
    fn empty() -> Self where Self: Sized {
        Default::default()
    }

    #[inline]
    fn uninit(size: usize) -> Result<Self, Self::ConstructorError> where Self: Sized {
        Ok(DataSlice::from_boxed_slice(DataBoxed::uninit(size)?.inner))
    }

    #[inline]
    fn filled(size: usize, byte: u8) -> Result<Self, Self::ConstructorError> where Self: Sized {
        Ok(DataSlice::from_boxed_slice(DataBoxed::filled(size, byte)?.inner))
    }

    #[inline]
    fn from_data_array<const SIZE: usize>(array: crate::array::DataArray<SIZE>) -> Result<Self, Self::ConstructorError> where Self: Sized {
        // Ok(
        //     unsafe {
        //         core::mem::transmute(
        //             #[cfg(feature = "allocator_api")] {
        //                 let slice: Box<[u8]> = Box::try_new(array.inner);
        //                 slice
        //             }
        //             #[cfg(not(feature = "allocator_api"))] {
        //                 let slice: Box<[u8]> = Box::new(array.inner);
        //                 slice
        //             }
        //         )
        //     }
        // )

        todo!()
    }
}

// unsafe impl crate::RawDataStructure for alloc::vec::Vec<u8> {
//     #[inline]
//     fn size(&self) -> usize {
//         <crate::slice::DataSlice as crate::RawDataStructure>::size(crate::slice::DataSlice::from_slice(self))
//     }

//     #[inline]
//     fn read_validity(&self, idx: usize, size: usize) -> Result<(), crate::idx::IdxError> {
//         <crate::slice::DataSlice as crate::RawDataStructure>::read_validity(crate::slice::DataSlice::from_slice(self), idx, size)
//     }

//     #[inline]
//     unsafe fn write_zeroes_unchecked(&mut self, idx: usize, size: usize) {
//         <crate::slice::DataSlice as crate::RawDataStructure>::write_zeroes_unchecked(crate::slice::DataSlice::from_slice_mut(self), idx, size)
//     }

//     #[inline]
//     unsafe fn write_ones_unchecked(&mut self, idx: usize, size: usize) {
//         <crate::slice::DataSlice as crate::RawDataStructure>::write_ones_unchecked(crate::slice::DataSlice::from_slice_mut(self), idx, size)
//     }

//     #[inline]
//     unsafe fn write_unsized_unchecked<T: ?Sized>(&mut self, idx: usize, value: *const core::mem::ManuallyDrop<T>) {
//         <crate::slice::DataSlice as crate::RawDataStructure>::write_unsized_unchecked(crate::slice::DataSlice::from_slice_mut(self), idx, value)
//     }

//     #[inline]
//     unsafe fn read_unchecked<T: Sized>(&self, idx: usize) -> *const T {
//         <crate::slice::DataSlice as crate::RawDataStructure>::read_unchecked(crate::slice::DataSlice::from_slice(self), idx)
//     }

//     #[inline]
//     unsafe fn read_mut_unchecked<T: Sized>(&mut self, idx: usize) -> *mut T {
//         <crate::slice::DataSlice as crate::RawDataStructure>::read_mut_unchecked(crate::slice::DataSlice::from_slice_mut(self), idx)
//     }

//     #[inline]
//     unsafe fn read_unsized_unchecked<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> *const T {
//         <crate::slice::DataSlice as crate::RawDataStructure>::read_unsized_unchecked(crate::slice::DataSlice::from_slice(self), idx, meta)
//     }

//     #[inline]
//     unsafe fn read_unsized_mut_unchecked<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> *mut T {
//         <crate::slice::DataSlice as crate::RawDataStructure>::read_unsized_mut_unchecked(crate::slice::DataSlice::from_slice_mut(self), idx, meta)
//     }

//     #[inline]
//     unsafe fn take_unchecked<T: Sized>(&self, idx: usize) -> T {
//         <crate::slice::DataSlice as crate::RawDataStructure>::take_unchecked(crate::slice::DataSlice::from_slice(self), idx)
//     }

//     #[inline]
//     unsafe fn clone_from_unchecked(&mut self, data: &Self) {
//         <crate::slice::DataSlice as crate::RawDataStructure>::clone_from_unchecked(
//             crate::slice::DataSlice::from_slice_mut(self),
//             crate::slice::DataSlice::from_slice(data),
//         )
//     }

//     type DataByte = u8;

//     #[inline]
//     unsafe fn get_at_idx(&self, idx: usize) -> Self::DataByte {
//         <crate::slice::DataSlice as crate::RawDataStructure>::get_at_idx(crate::slice::DataSlice::from_slice(self), idx)
//     }

//     #[inline]
//     unsafe fn set_at_idx(&mut self, idx: usize, byte: Self::DataByte) {
//         <crate::slice::DataSlice as crate::RawDataStructure>::set_at_idx(crate::slice::DataSlice::from_slice_mut(self), idx, byte)
//     }
// }
