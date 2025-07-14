
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
