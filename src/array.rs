
use crate::{
    idx,
    slice::DataSlice,
};
#[cfg(feature = "ptr_metadata")]
use crate::GetSizeOf;
use core::mem::ManuallyDrop;

#[must_use]
#[repr(transparent)]
pub struct DataArray<const SIZE: usize> {
    pub(crate) inner: [u8; SIZE]
}

impl<const SIZE: usize> DataArray<SIZE> {
    /// Constructs a new [Data] structure without touching the underling data.
    /// 
    /// This method is safe because reading in it'self from the data structure is
    /// an unsafe operation, this function marking that the udnerlyign data does
    /// not matter at all when it starts.
    #[inline] pub const fn uninit() -> DataArray<SIZE> {
        DataArray {
            inner: unsafe { core::mem::MaybeUninit::uninit().assume_init() }
        }
    }

    /// Constructs a new [Data] structure filled with `0`'s.
    #[inline] pub const fn zeroed() -> DataArray<SIZE> {
        DataArray {
            inner: [0x00; SIZE]
        }
    }

    /// Constructs a new [Data] structure filled with `1`'s.
    #[inline] pub const fn filled(byte: u8) -> DataArray<SIZE> {
        DataArray {
            inner: [byte; SIZE]
        }
    }

    /// Constructs a new [Data] structure with the given array as a data preset.
    #[inline] pub const fn from_array(array: [u8; SIZE]) -> DataArray<SIZE> {
        DataArray {
            inner: array
        }
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
    /// If you want to store a [?Sized](Sized) value use [write_unsized](Data::write_unsized)
    /// 
    /// # SAFETY
    /// Make sure for all the data inside to follow the
    /// ownership and borrowing rules and guarantees.
    pub const unsafe fn write<T: Sized>(&mut self, idx: usize, value: ManuallyDrop<T>) -> Result<(), (ManuallyDrop<T>, idx::IdxError)> {
        let type_size: usize = core::mem::size_of_val(&value);

        if match idx.checked_add(type_size) {
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err((value, idx::IdxError { idx, data_size: SIZE, type_size }))
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
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: SIZE, type_size: size })
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
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: SIZE, type_size: size })
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
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: SIZE, type_size })
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
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: SIZE, type_size: core::mem::size_of::<T>() })
        }

        Ok(
            unsafe {
                // SAFETY: The addr of this ptr + idx is guaranteed to be in
                // the data region given to self.inner, which is guaranteed
                // to be in a valid address by the fact that is exists.
                (&self.inner as *const u8).add(idx).cast()
            }
        )
    }

    /// Returns a mutable pointer to the specified data region.
    /// 
    /// The pointer is guaranteed to ne non-null.
    // Not using NonNull is intentional
    pub const fn read_mut<T: Sized>(&mut self, idx: usize) -> Result<*mut T, idx::IdxError> {
        if match idx.checked_add(core::mem::size_of::<T>()) {
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: SIZE, type_size: core::mem::size_of::<T>() })
        }

        Ok(
            unsafe {
                // SAFETY: The addr of this ptr + idx is guaranteed to be in
                // the data region given to self.inner, which is guaranteed
                // to be in a valid address by the fact that is exists.
                (&mut self.inner as *mut u8).add(idx).cast()
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
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: SIZE, type_size: meta.size() })
        }

        Ok(
            core::ptr::from_raw_parts(
                &self.inner as *const u8,
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
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: SIZE, type_size: meta.size() })
        }

        Ok(
            core::ptr::from_raw_parts_mut(
                &mut self.inner as *mut u8,
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
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: SIZE, type_size: core::mem::size_of::<T>() })
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
                (&self.inner as *const u8).add(idx)
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
            Some(size) => size >= SIZE,
            None => true,
        } {
            return Err(idx::IdxError { idx, data_size: SIZE, type_size: core::mem::size_of::<T>() })
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
                    (&mut self.inner as *mut u8).add(idx).cast(),
                    ManuallyDrop::into_inner(value)
                )
            }
        )
    }

    #[inline]
    pub const fn idx_const(&self, start: core::ops::Bound<usize>, end: core::ops::Bound<usize>) -> Option<&DataSlice> {
        self.deref().idx_const(start, end)
    }

    #[inline]
    pub const fn idx_mut_const(&mut self, start: core::ops::Bound<usize>, end: core::ops::Bound<usize>) -> Option<&mut DataSlice> {
        self.deref_mut().idx_mut_const(start, end)
    }

    #[inline]
    pub const fn deref(&self) -> &crate::slice::DataSlice {
        crate::slice::DataSlice::from_slice(&self.inner)
    }

    #[inline]
    pub const fn deref_mut(&mut self) -> &mut crate::slice::DataSlice {
        crate::slice::DataSlice::from_slice_mut(&mut self.inner)
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
        match self.idx(index) {
            Some(slice) => slice,
            None => panic!("Index out of bounds!"),
        }
    }
}

impl<I: idx::Idx, const SIZE: usize> core::ops::IndexMut<I> for DataArray<SIZE> {
    fn index_mut(&mut self, index: I) -> &mut DataSlice {
        match self.idx_mut(index) {
            Some(slice) => slice,
            None => panic!("Index out of bounds!"),
        }
    }
}
