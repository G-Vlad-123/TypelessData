
/*!
This module provides the [`DataArray`] data structure and all it's associated
functions, methods and items.

[`DataArray`] is made to be usable any context.

Under the surface a [`DataArray`] is just an array of bytes with the special designation
that it is a region of memory where each peace of data and it's type is not determined by
the compiler.
 */

use crate::{
    idx,
    slice::DataSlice,
};
use core::mem::ManuallyDrop;

/// A typeless chunk of data.
/// 
/// This provides most of the `const` functionality of the crate.
/// 
/// This struct is just an array of bytes underneeth the hood.
#[must_use]
#[repr(transparent)]
pub struct DataArray<const SIZE: usize> {
    pub(crate) inner: [u8; SIZE]
}

impl<const SIZE: usize> DataArray<SIZE> {
    /// Constructs a new [`DataArray`] structure without touching the underling data.
    /// 
    /// This method is safe because reading in it'self from the data structure is
    /// an unsafe operation, this function marking that the underlying data does
    /// not matter at all when it starts.
    #[inline] pub const fn uninit() -> DataArray<SIZE> {
        DataArray {
            inner: unsafe { core::mem::MaybeUninit::uninit().assume_init() }
        }
    }

    /// Constructs a new [`DataArray`] structure filled with `0`'s.
    #[inline] pub const fn zeroed() -> DataArray<SIZE> {
        DataArray {
            inner: [0x00; SIZE]
        }
    }

    /// Constructs a new [`DataArray`] structure filled with whatever byte you give.
    #[inline] pub const fn filled(byte: u8) -> DataArray<SIZE> {
        DataArray {
            inner: [byte; SIZE]
        }
    }

    /// Constructs a new [`DataArray`] structure with the given array as a data preset.
    #[inline] pub const fn from_array(array: [u8; SIZE]) -> DataArray<SIZE> {
        DataArray {
            inner: array
        }
    }
    
    /// Constructs a new [`DataArray`] structure with the given slice as a data preset.
    /// 
    /// Will return an error if the sizes the the slice and teh requested [`DataArray`] do not match.
    pub const fn try_from_slice(slice: &[u8]) -> Result<Self, DiferentSizesError<SIZE>> {
        if slice.len() != SIZE {
            return Err(
                DiferentSizesError { gotten_size: slice.len() }
            )
        }
        
        let mut data: DataArray<SIZE> = DataArray::uninit();
        data.inner.copy_from_slice(slice);
        Ok(data)
    }
    
    /// Constructs a new [`DataArray`] structure with the given [`DataSlice`] as a data preset.
    /// 
    /// Will return an error if the sizes the the [`DataSlice`] and teh requested [`DataArray`] do not match.
    pub const fn try_from_data_slice(slice: &DataSlice) -> Result<Self, DiferentSizesError<SIZE>> {
        if slice.size() != SIZE {
            return Err(
                DiferentSizesError { gotten_size: slice.size() }
            )
        }
        
        let mut data: DataArray<SIZE> = DataArray::uninit();
        data.inner.copy_from_slice(&slice.inner);
        Ok(data)
    }

    /// Clones the entire chunk of data.
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    pub const unsafe fn clone(&self) -> DataArray<SIZE> {
        let mut data = DataArray::uninit();
        let mut idx: usize = 0;
        
        while idx < SIZE {
            data.inner[idx] = self.inner[idx];
            idx += 1;
        }

        data
    }

    /// Writes the given value at the given index.
    /// 
    /// If you want to store a [?Sized](Sized) value use [write_unsized](DataArray::write_unsized)
    /// 
    /// # ERRORS
    /// Will return an error if the write function catches
    /// it'self trying to write in a memory region that is
    /// not assigned to the data structure.
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    #[inline]
    pub const unsafe fn write<T: Sized>(&mut self, idx: usize, value: ManuallyDrop<T>) -> Result<(), (ManuallyDrop<T>, idx::IdxError)> {
        self.deref_mut().write(idx, value)
    }

    /// Writes the given value at the given index.
    /// 
    /// If you want to store a [?Sized](Sized) value use [write_unsized](DataArray::write_unsized)
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure no data is written to a region outside of the specified data structure
    #[inline]
    pub const unsafe fn write_unchecked<T: Sized>(&mut self, idx: usize, value: ManuallyDrop<T>) {
        self.deref_mut().write_unchecked(idx, value)
    }

    /// Fills with `0`'s the specified bytes
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    #[inline]
    pub const unsafe fn write_zeroes(&mut self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        self.deref_mut().write_zeroes(idx, size)
    }

    /// Fills with `1`'s the specified bytes
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    #[inline]
    pub const unsafe fn write_ones(&mut self, idx: usize, size: usize) -> Result<(), idx::IdxError> {
        self.deref_mut().write_ones(idx, size)
    }

    /// Writes the given value at the given index.
    /// 
    /// This method performs a shallow copy (the)
    /// 
    /// This method takes ownership of T, the reason why
    /// a box is not used is to avoid needless heap alocations.
    /// 
    /// If you want to store a sized value it
    /// is recomended to use [write](DataArray::write) instead.
    /// 
    /// # PANICS
    /// Will panic if a null pointer is given.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure that the value is not used again after being given to this funtion
    /// (eg: using [`mem::forget`](core::mem::forget) or moving the value into a [ManuallyDrop])
    #[inline]
    pub const unsafe fn write_unsized<T: ?Sized>(&mut self, idx: usize, value: *const ManuallyDrop<T>) -> Result<(), idx::IdxError> {
        self.deref_mut().write_unsized(idx, value)
    }

    /// Writes the given value at the given index.
    /// 
    /// This method performs a shallow copy (the)
    /// 
    /// This method takes ownership of T, the reason why
    /// a box is not used is to avoid needless heap alocations.
    /// 
    /// If you want to store a sized value it
    /// is recomended to use [write](Data::write) instead.
    /// 
    /// # PANICS
    /// Will panic if a null pointer is given.
    /// 
    /// # SAFETY
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure that the value is not used again after being given to this funtion
    /// (eg: using [`mem::forget`](core::mem::forget) or moving the value into a [`ManuallyDrop`])
    /// - Make sure no data is written to a region outside of the specified data structure
    #[inline]
    pub const unsafe fn write_unsized_unchecked<T: ?Sized>(&mut self, idx: usize, value: *const ManuallyDrop<T>) {
        self.deref_mut().write_unsized_unchecked(idx, value)
    }

    /// Returns a pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    // Not using NonNull is intentional
    #[inline]
    pub const fn read<T: Sized>(&self, idx: usize) -> Result<*const T, idx::IdxError> {
        self.deref().read(idx)
    }

    /// Returns a pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    // Not using NonNull is intentional (NonNull is *mut, not *const)
    #[inline]
    pub const unsafe fn read_unchecked<T: Sized>(&self, idx: usize) -> *const T {
        self.deref().read_unchecked(idx)
    }

    /// Returns a mutable pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    // Not using NonNull is intentional
    #[inline]
    pub const fn read_mut<T: Sized>(&mut self, idx: usize) -> Result<*mut T, idx::IdxError> {
        self.deref_mut().read_mut(idx)
    }

    /// Returns a mutable pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    // Not using NonNull is intentional (consistancy with read)
    #[inline]
    pub const unsafe fn read_mut_unchecked<T: Sized>(&mut self, idx: usize) -> *mut T {
        self.deref_mut().read_mut_unchecked(idx)
    }

    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_unchecked](DataSlice::read_unchecked) instead.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    #[inline]
    pub const unsafe fn read_unsized_unchecked<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> *const T
    {
        self.deref().read_unsized_unchecked(idx, meta)
    }
    
    /// Returns a pointer to the specified data region with the provided metadata.
    /// 
    /// If you know T is sized use [read_mut_unchecked](DataSlice::read_mut_unchecked) instead.
    /// 
    /// # SAFETY
    /// Make sure data isn't read from outside the data structure
    #[cfg(feature = "ptr_metadata")]
    #[allow(private_bounds)]
    #[inline]
    pub const unsafe fn read_unsized_mut_unchecked<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> *mut T
    {
        self.deref_mut().read_unsized_mut_unchecked(idx, meta)
    }

    /// Takes the value from the specified region.
    /// 
    /// Note: This does NOT zero out the specified region
    /// 
    /// # Safety
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the data gotten from inside is a valid T
    #[inline]
    pub const unsafe fn take<T: Sized>(&self, idx: usize) -> Result<T, idx::IdxError> {
        self.deref().take(idx)
    }

    /// Takes the value from the specified region and writes a new value in it's palce.
    /// 
    /// # Safety
    /// - Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    /// - Make sure the data gotten from inside is a valid T
    #[inline]
    pub const unsafe fn replace<T: Sized>(&mut self, idx: usize, value: ManuallyDrop<T>) -> Result<T, idx::IdxError> {
        self.deref_mut().replace(idx, value)
    }

    #[inline]
    /// Get's a subslice of the data structure in a const context.
    pub const fn get_const(&self, start: core::ops::Bound<usize>, end: core::ops::Bound<usize>) -> Option<&DataSlice> {
        self.deref().get_const(start, end)
    }

    #[inline]
    /// Get's a mutable subslice of the data structure in a const context.
    pub const fn get_mut_const(&mut self, start: core::ops::Bound<usize>, end: core::ops::Bound<usize>) -> Option<&mut DataSlice> {
        self.deref_mut().get_mut_const(start, end)
    }

    /// The const version of the [Deref](core::ops::Deref) trait.
    #[inline]
    pub const fn deref(&self) -> &crate::slice::DataSlice {
        crate::slice::DataSlice::from_slice(&self.inner)
    }

    /// The const version of the [DerefMut](core::ops::DerefMut) trait.
    #[inline]
    pub const fn deref_mut(&mut self) -> &mut crate::slice::DataSlice {
        crate::slice::DataSlice::from_slice_mut(&mut self.inner)
    }
}

use core::convert::TryFrom;

/// The error given when converting from a slice or a [`DataSlice`] into a [`DataArray`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiferentSizesError<const SIZE: usize> { gotten_size: usize }

impl<const SIZE: usize> core::error::Error for DiferentSizesError<SIZE> {}
impl<const SIZE: usize> core::fmt::Display for DiferentSizesError<SIZE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Failed to turn slice with size `{got}` into a DataArray of size {SIZE} because they are of diferent sizes.", got = self.gotten_size)
    }
}

impl<const SIZE: usize> From<[u8; SIZE]> for DataArray<SIZE> {
    #[inline] fn from(inner: [u8; SIZE]) -> Self {
        DataArray { inner }
    }
}

impl<const SIZE: usize> TryFrom<&[u8]> for DataArray<SIZE> {
    type Error = DiferentSizesError<SIZE>;

    #[inline] fn try_from(slice: &[u8]) -> Result<Self, DiferentSizesError<SIZE>> {
        if slice.len() != SIZE {
            return Err(
                DiferentSizesError { gotten_size: slice.len() }
            )
        }
        
        let mut data: DataArray<SIZE> = DataArray::uninit();
        data.inner.copy_from_slice(slice);
        Ok(data)
    }
}

impl<const SIZE: usize> TryFrom<&DataSlice> for DataArray<SIZE> {
    type Error = DiferentSizesError<SIZE>;

    #[inline] fn try_from(slice: &DataSlice) -> Result<Self, DiferentSizesError<SIZE>> {
        if slice.size() != SIZE {
            return Err(
                DiferentSizesError { gotten_size: slice.size() }
            )
        }
        
        let mut data: DataArray<SIZE> = DataArray::uninit();
        data.inner.copy_from_slice(&slice.inner);
        Ok(data)
    }
}

impl<const SIZE: usize> core::ops::Deref for DataArray<SIZE> {
    type Target = crate::slice::DataSlice;

    #[inline] fn deref(&self) -> &Self::Target {
        self.deref()
    }
}

impl<const SIZE: usize> core::ops::DerefMut for DataArray<SIZE> {
    #[inline] fn deref_mut(&mut self) -> &mut Self::Target {
        self.deref_mut()
    }
}

/// Constructs a new [Data] structure using the [uninit](Data::uninit) constructor.
impl<const SIZE: usize> Default for DataArray<SIZE> {
    #[inline] fn default() -> Self {
        DataArray::uninit()
    }
}

impl<const SIZE: usize> core::fmt::Debug for DataArray<SIZE> {
    #[inline] fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <crate::slice::DataSlice as core::fmt::Debug>::fmt(self, f)
    }
}

impl<'data, const SIZE: usize> IntoIterator for &'data DataArray<SIZE> {
    type Item = u8;
    type IntoIter = core::iter::Copied<core::slice::Iter<'data, u8>>;

    #[inline] fn into_iter(self) -> Self::IntoIter {
        self.deref().inner.iter().copied()
    }
}

impl<'data, const SIZE: usize> IntoIterator for &'data mut DataArray<SIZE> {
    type Item = &'data mut u8;
    type IntoIter = core::slice::IterMut<'data, u8>;

    #[inline] fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().inner.iter_mut()
    }
}

impl<I: idx::Idx, const SIZE: usize> core::ops::Index<I> for DataArray<SIZE> {
    type Output = DataSlice;

    fn index(&self, index: I) -> &DataSlice {
        match self.get(index) {
            Some(slice) => slice,
            None => panic!("Index out of bounds!"),
        }
    }
}

impl<I: idx::Idx, const SIZE: usize> core::ops::IndexMut<I> for DataArray<SIZE> {
    fn index_mut(&mut self, index: I) -> &mut DataSlice {
        match self.get_mut(index) {
            Some(slice) => slice,
            None => panic!("Index out of bounds!"),
        }
    }
}

/// Constructs a new data array structure without touching the underlying data.
/// 
/// Go to [DataArray::uninit] for more infomration.
#[inline(always)]
pub const fn uninit<const SIZE: usize>() -> DataArray<SIZE> {
    DataArray::uninit()
}

/// Constructs a new data array from a given sized value.
#[cfg(feature = "generic_const_exprs")]
pub const fn from_copy<T: Copy>(value: T) -> DataArray<{core::mem::size_of::<T>()}> {
    let mut data = uninit();
    unsafe {
        // SAFETY:
        // - The value is copy, there for the bits can be copied perfectly
        // - The data structure is the exact same size as the written data
        //   And it's written starting from the vary first byte
        //   Making sure it fits perfectly inside the data structure
        data.write_unchecked(0, ManuallyDrop::new(value));
    }
    data
}

/// Constructs a new data array from a given sized value.
/// 
/// # SAFETY
/// Make sure for all the data inside to follow the
/// ownership and borrowing rules and guarantees.
#[cfg(feature = "generic_const_exprs")]
pub const unsafe fn from_sized<T: Sized>(value: T) -> DataArray<{core::mem::size_of::<T>()}> {
    let mut data = uninit();
    unsafe {
        // SAFETY:
        // - It is up to the caller to uphold the safety of this call
        // - The data structure is the exact same size as the written data
        //   And it's written starting from the vary first byte
        //   Making sure it fits perfectly inside the data structure
        data.write_unchecked(0, ManuallyDrop::new(value));
    }
    data
}

// /// Constructs a new data array from a given sized value.
// /// 
// /// # SAFETY
// /// Make sure for all the data inside to follow the
// /// ownership and borrowing rules and guarantees.
// pub const unsafe fn try_from_unsized<const SIZE: usize, T: ?Sized>(value: *mut T) -> Option<DataArray<SIZE>> {
//     // let type_size: usize = core::mem::size_of_val(value);
//     let mut data = uninit();
//     unsafe {
//         // SAFETY:
//         // - It is up to the caller to uphold the safety of this call
//         // - The data structure is the exact same size as the written data
//         //   And it's written starting from the vary first byte
//         //   Making sure it fits perfectly inside the data structure
//         data.write_unchecked(0, ManuallyDrop::new(value));
//     }
//     Some(data)
// }

// /// Constructs a new data structure using a function to set the starting bytes.
// #[inline]
// pub fn from_fn<const SIZE: usize>(f: impl FnMut(usize) -> u8) -> DataArray<SIZE> {
//     DataArray { inner: core::array::from_fn(f) }
// }

unsafe impl<const SIZE: usize> crate::RawDataStructure for DataArray<SIZE> {
    #[inline] fn size(&self) -> usize { SIZE }

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
        for (idx, byte) in self.iter_mut().enumerate() {
            *byte = data.inner[idx]
        }
    }
    
    #[inline]
    unsafe fn write_zeroes_unchecked(&mut self, idx: usize, size: usize) {
        self.deref_mut().write_zeroes_unchecked(idx, size)
    }

    #[inline]
    unsafe fn write_ones_unchecked(&mut self, idx: usize, size: usize) {
        self.deref_mut().write_ones_unchecked(idx, size)
    }

    #[inline]
    unsafe fn write_unsized_unchecked<T: ?Sized>(&mut self, idx: usize, value: *const core::mem::ManuallyDrop<T>) {
        self.deref_mut().write_unsized_unchecked(idx, value)
    }

    #[inline]
    unsafe fn read_unchecked<T: Sized>(&self, idx: usize) -> *const T {
        self.deref().read_unchecked(idx)
    }

    #[inline]
    unsafe fn read_mut_unchecked<T: Sized>(&mut self, idx: usize) -> *mut T {
        self.deref_mut().read_mut_unchecked(idx)
    }

    #[inline]
    #[cfg(feature = "ptr_metadata")]
    unsafe fn read_unsized_unchecked<T: ?Sized + core::ptr::Pointee>(&self, idx: usize, meta: T::Metadata) -> *const T {
        self.deref().read_unsized_unchecked(idx, meta)
    }

    #[inline]
    #[cfg(feature = "ptr_metadata")]
    unsafe fn read_unsized_mut_unchecked<T: ?Sized + core::ptr::Pointee>(&mut self, idx: usize, meta: T::Metadata) -> *mut T {
        self.deref_mut().read_unsized_mut_unchecked(idx, meta)
    }

    #[inline]
    unsafe fn take_unchecked<T: Sized>(&self, idx: usize) -> T {
        self.deref().take_unchecked(idx)
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

impl<const SIZE: usize> crate::DataStructureSlice for DataArray<SIZE> {
    unsafe fn get_unchecked(&self, idx: impl idx::Idx) -> *const crate::slice::DataSlice {
        self.deref().get_unchecked(idx)
    }

    unsafe fn get_mut_unchecked(&mut self, idx: impl idx::Idx) -> *mut crate::slice::DataSlice {
        self.deref_mut().get_mut_unchecked(idx)
    }
}
