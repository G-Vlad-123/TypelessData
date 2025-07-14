
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

pub mod array;
pub mod slice;
#[cfg(feature = "alloc")]
pub mod boxed;

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

#[cfg(test)]
mod tests;

// TODO:
//   - Remove the need for serde_derive feature
//   - Add Serialize and Deserialize for Data
