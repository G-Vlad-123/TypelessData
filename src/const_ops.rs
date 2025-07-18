
use crate::idx;
use crate::GetSizeOf;

/// Checks weather an index at a surtun location with a surtun size is readable.
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
/// safe to use an unsafe read/write function that asks for the read/write data to
/// not be from outside the data structure as long as all the other
/// safety requirments (if any) are also satisfied.
pub const fn validity(slice: &[u8], idx: usize, size: usize) -> Result<(), idx::IdxError> {
    if match idx.checked_add(size) {
        Some(size) => size < slice.len(),
        None => false,
    } {
        Ok(())
    } else {
        Err(idx::IdxError { idx, data_size: slice.len(), type_size: size })
    }
}

#[doc = include_str!("doc/write.md")]
/// [write_unsized]: write_unsized
pub const unsafe fn write<T: Sized>(slice: &mut [u8], idx: usize, value: core::mem::ManuallyDrop<T>) -> Result<(), (core::mem::ManuallyDrop<T>, idx::IdxError)> {
    if let Err(err) = validity(slice, idx, core::mem::size_of::<T>()) {
        return Err((value, err));
    }

    write_unchecked(slice, idx, value);
    Ok(())
}

#[doc = include_str!("doc/write_unchecked.md")]
/// [write_unsized_unchecked]: write_unsized_unchecked
#[inline]
pub const unsafe fn write_unchecked<T: Sized>(slice: &mut [u8], idx: usize, value: core::mem::ManuallyDrop<T>) {
    write_unsized_unchecked(slice, idx, &value)
}

/// Fills with `0`'s the specified bytes
/// 
/// # SAFETY
/// Make sure for all the data inside to follow the
/// ownership and borrowing rules and guarantees.
pub const unsafe fn write_zeroes(slice: &mut [u8], idx: usize, size: usize) -> Result<(), idx::IdxError> {
    if let Err(err) = validity(slice, idx, size) {
        return Err(err);
    }
    write_zeroes_unchecked(slice, idx, size);
    Ok(())
}

/// Fills with `0`'s the specified bytes
/// 
/// # SAFETY
/// - Make sure for all the data inside to follow the
/// ownership and borrowing rules and guarantees.
/// - Make sure no data is written to a region outside of the specified data structure.
/// - Make sure idx + size doesn't overflow.
pub const unsafe fn write_zeroes_unchecked(slice: &mut [u8], idx: usize, size: usize) {
    let mut at: usize = idx;
    let stop = idx.unchecked_add(size);
    while at < stop {
        slice[at] = 0x00;
        at += 1;
    }
}

/// Fills with `1`'s the specified bytes
/// 
/// # SAFETY
/// Make sure for all the data inside to follow the
/// ownership and borrowing rules and guarantees.
pub const unsafe fn write_ones(slice: &mut [u8], idx: usize, size: usize) -> Result<(), idx::IdxError> {
    if let Err(err) = validity(slice, idx, size) {
        return Err(err);
    }
    write_ones_unchecked(slice, idx, size);
    Ok(())
}

/// Fills with `1`'s the specified bytes
/// 
/// # SAFETY
/// - Make sure for all the data inside to follow the
/// ownership and borrowing rules and guarantees.
/// - Make sure no data is written to a region outside of the specified data structure.
/// - Make sure idx + size doesn't overflow.
pub const unsafe fn write_ones_unchecked(slice: &mut [u8], idx: usize, size: usize) {
    let mut at: usize = idx;
    let stop = idx.unchecked_add(size);
    while at < stop {
        slice[at] = 0xFF;
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
/// If you want to store a sized value it
/// is recomended to use [write] instead.
/// 
/// # PANICS
/// Will panic if a null pointer is given.
/// 
/// # SAFETY
/// - Make sure for all the data inside to follow the
/// ownership and borrowing rules and guarantees.
/// - Make sure that the value is not used again after being given to this funtion
/// (eg: using [`mem::forget`](core::mem::forget) or moving the value into a [`ManuallyDrop`](core::mem::ManuallyDrop))
pub const unsafe fn write_unsized<T: ?Sized>(slice: &mut [u8], idx: usize, value: *const T) -> Result<(), idx::IdxError> {
    if let Err(err) = validity(
        slice,
        idx,
        core::mem::size_of_val::<T>(
            match value.as_ref() {
                Some(some) => some,
                None => unimplemented!(),
            }
        )
    ) {
        return Err(err);
    }

    write_unsized_unchecked(slice, idx, value);

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
/// is recomended to use [write_unchecked] instead.
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
pub const unsafe fn write_unsized_unchecked<T: ?Sized>(slice: &mut [u8], idx: usize, value: *const T) {
    let type_size: usize = core::mem::size_of_val::<T>(
        match value.as_ref() {
            Some(some) => some,
            None => unimplemented!(),
        }
    );
    
    let ptr: *const u8 = value.cast();
    let mut at: usize = 0;

    while at < type_size {
        slice[at + idx] = unsafe {
            *ptr.add(at)
        };
        at += 1;
    }
}

/// Returns a pointer to the specified data region.
/// 
/// The pointer is guaranteed to be non-null.
// Not using NonNull is intentional
pub const fn read<T: Sized>(slice: &[u8], idx: usize) -> Result<*const T, crate::idx::IdxError> {
    if let Err(err) = validity(slice, idx, core::mem::size_of::<T>()) {
        return Err(err);
    }

    Ok(
        unsafe {
            // SAFETY: the validity check ensures the data will not be read outside the data structure.
            read_unchecked::<T>(slice, idx)
        }
    )
}

/// Returns a refrence to the specified data region.
/// 
/// # SAFETY
/// - Make sure the data is aligned
/// - Make sure the data is valid
// Not using NonNull is intentional
pub const unsafe fn read_ref<T: Sized>(slice: &[u8], idx: usize) -> Result<&T, crate::idx::IdxError> {
    match read::<T>(slice, idx) {
        Ok(ptr) => Ok(
            unsafe {
                ptr.as_ref() // SAFETY: The caller must uphold the safety contract.
                   .unwrap_unchecked() // SAFETY: read can never return a null ptr.
            }
        ),
        Err(err) => Err(err),
    }
}

/// Returns a pointer to the specified data region.
/// 
/// The pointer is guaranteed to ne non-null.
/// 
/// # SAFETY
/// Make sure data isn't read from outside the data structure
// Not using NonNull is intentional (NonNull is *mut, not *const)
pub const unsafe fn read_unchecked<T: Sized>(slice: &[u8], idx: usize) -> *const T {
    unsafe {
        // SAFETY: Must be upheld by the caller.
        (slice as *const [u8]).cast::<T>().add(idx)
    }
}

/// Returns a refrence to the specified data region.
/// 
/// # SAFETY
/// - Make sure data isn't read from outside the data structure
/// - Make sure the data is aligned
/// - Make sure the data is valid
pub const unsafe fn read_ref_unchecked<T: Sized>(slice: &[u8], idx: usize) -> &T {
    unsafe {
        read_unchecked::<T>(slice, idx) // SAFETY: The caller must uphold the safety contract.
            .as_ref() // SAFETY: The caller must uphold the safety contract.
            .unwrap_unchecked() // SAFETY: read can never return a null ptr.
    }
}

/// Returns a mutable pointer to the specified data region.
/// 
/// The pointer is guaranteed to ne non-null.
// Not using NonNull is intentional
pub const fn read_mut<T: Sized>(slice: &mut [u8], idx: usize) -> Result<*mut T, crate::idx::IdxError> {
    match validity(slice, idx, core::mem::size_of::<T>()) {
        Err(err) => Err(err),
        Ok(()) => Ok(
            // SAFETY: The data will always be from within the data structure
            unsafe { read_mut_unchecked::<T>(slice, idx) }
        )
    }
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
pub const unsafe fn read_ref_mut<T: Sized>(slice: &mut [u8], idx: usize) -> Result<&mut T, crate::idx::IdxError> {
    match read_mut::<T>(slice, idx) {
        Ok(ptr) => Ok(
            unsafe {
                ptr.as_mut() // SAFETY: The caller msut uphold the safety contract.
                   .unwrap_unchecked() // SAFETY: read can never return a null ptr.
            }
        ),
        Err(err) => Err(err),
    }
}

/// Returns a mutable pointer to the specified data region.
/// 
/// The pointer is guaranteed to ne non-null.
/// 
/// # SAFETY
/// Make sure data isn't read from outside the data structure
// Not using NonNull is intentional (consistancy with read)
pub const unsafe fn read_mut_unchecked<T: Sized>(slice: &mut [u8], idx: usize) -> *mut T {
    unsafe {
        // SAFETY: Must be upheld by the caller.
        (slice as *mut [u8]).cast::<T>().add(idx)
    }
}

/// Returns a mutable pointer to the specified data region.
/// 
/// The pointer is guaranteed to ne non-null.
/// 
/// # SAFETY
/// - Make sure data isn't read from outside the data structure
/// - Make sure the data is aligned
/// - Make sure the data is valid
/// - Make sure there is only one refrence to the targeted value
pub const unsafe fn read_ref_mut_unchecked<T: Sized>(slice: &mut [u8], idx: usize) -> &mut T {
    unsafe {
        read_mut_unchecked::<T>(slice, idx) // SAFETY: The caller must uphold the safety contract.
            .as_mut() // SAFETY: The caller msut uphold the safety contract.
            .unwrap_unchecked() // SAFETY: read can never return a null ptr.
    }
}

/// Returns a pointer to the specified data region with the provided metadata.
/// 
/// If you know T is sized use [read](RawDataStructure::read) instead.
#[cfg(feature = "ptr_metadata")]
#[allow(private_bounds)]
pub fn read_unsized<T: ?Sized + core::ptr::Pointee>(slice: &[u8], idx: usize, meta: T::Metadata) -> Result<*const T, idx::IdxError>
where T::Metadata: GetSizeOf<T> {
    match validity(slice, idx, meta.size()) {
        Err(err) => Err(err),
        Ok(()) => Ok(
            // SAFETY: The data will always be from within the data structure
            unsafe { read_unsized_unchecked(slice, idx, meta) }
        )
    }
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
pub unsafe fn read_unsized_ref<T: ?Sized + core::ptr::Pointee>(slice: &[u8], idx: usize, meta: T::Metadata) -> Result<&T, idx::IdxError>
where T::Metadata: crate::GetSizeOf<T> {
    match read_unsized::<T>(slice, idx, meta) {
        Err(err) => Err(err),
        Ok(ptr) => Ok(
            unsafe {
                ptr.as_ref() // SAFETY: The caller msut uphold the safety contract.
                    .unwrap_unchecked() // SAFETY: read can never return a null ptr.
            }
        )
    }
}

/// Returns a pointer to the specified data region with the provided metadata.
/// 
/// If you know T is sized use [read_unchecked] instead.
/// 
/// # SAFETY
/// Make sure data isn't read from outside the data structure
#[cfg(feature = "ptr_metadata")]
#[allow(private_bounds)]
pub const unsafe fn read_unsized_unchecked<T: ?Sized + core::ptr::Pointee>(slice: &[u8], idx: usize, meta: T::Metadata) -> *const T {
    core::ptr::from_raw_parts(
        unsafe {
            // SAFETY: The safety must be upheld by the caller.
            (slice as *const [u8]).cast::<u8>().add(idx)
        },
        meta,
    )
}

/// Returns a pointer to the specified data region with the provided metadata.
/// 
/// If you know T is sized use [read_ref_unchecked](RawDataStructure::read_ref_unchecked) instead.
/// 
/// # SAFETY
/// Make sure data isn't read from outside the data structure
#[cfg(feature = "ptr_metadata")]
#[allow(private_bounds)]
pub const unsafe fn read_unsized_ref_unchecked<T: ?Sized + core::ptr::Pointee>(slice: &[u8], idx: usize, meta: T::Metadata) -> &T {
    read_unsized_unchecked::<T>(slice, idx, meta)
        .as_ref()
        .unwrap_unchecked()
}

/// Returns a pointer to the specified data region with the provided metadata.
/// 
/// If you know T is sized use [read_mut](RawDataStructure::read_mut) instead.
#[cfg(feature = "ptr_metadata")]
#[allow(private_bounds)]
pub fn read_unsized_mut<T: ?Sized + core::ptr::Pointee>(slice: &mut [u8], idx: usize, meta: T::Metadata) -> Result<*mut T, idx::IdxError>
where T::Metadata: crate::GetSizeOf<T> {
    validity(slice, idx, meta.size())?;
    
    Ok(
        // SAFETY: The data will always be from within the data structure
        unsafe { read_unsized_mut_unchecked(slice, idx, meta) }
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
pub unsafe fn read_unsized_ref_mut<T: ?Sized + core::ptr::Pointee>(slice: &mut [u8], idx: usize, meta: T::Metadata) -> Result<&mut T, idx::IdxError>
where T::Metadata: crate::GetSizeOf<T> {
    match read_unsized_mut::<T>(slice, idx, meta) {
        Err(err) => Err(err),
        Ok(ptr) => Ok(
            unsafe {
                ptr.as_mut() // SAFETY: The caller must uphold this safety contract
                   .unwrap_unchecked() // SAFETY: the ptr can not be null
            }
        )
    }
}

/// Returns a pointer to the specified data region with the provided metadata.
/// 
/// If you know T is sized use [read_mut_unchecked](RawDataStructure::read_mut_unchecked) instead.
/// 
/// # SAFETY
/// Make sure data isn't read from outside the data structure
#[cfg(feature = "ptr_metadata")]
#[allow(private_bounds)]
pub const unsafe fn read_unsized_mut_unchecked<T: ?Sized + core::ptr::Pointee>(slice: &mut [u8], idx: usize, meta: T::Metadata) -> *mut T {
    core::ptr::from_raw_parts_mut(
        unsafe {
            // SAFETY: The safety must be upheld by the caller.
            (slice as *mut [u8]).cast::<u8>().add(idx)
        },
        meta,
    )
}

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
pub const unsafe fn read_unsized_ref_mut_unchecked<T: ?Sized + core::ptr::Pointee>(slice: &mut [u8], idx: usize, meta: T::Metadata) -> &mut T {
    read_unsized_mut_unchecked::<T>(slice, idx, meta) // SAFETY: Up to the caller to uphold this safety contract
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
pub const unsafe fn take<T: Sized>(slice: &[u8], idx: usize) -> Result<T, idx::IdxError> {
    match validity(slice, idx, core::mem::size_of::<T>()) {
        Err(err) => Err(err),
        Ok(()) => Ok(take_unchecked(slice, idx)),
    }
}

/// Takes the value from the specified region.
/// 
/// Note: This does NOT zero out the specified region
/// 
/// # Safety
/// - Make sure for all the data inside to follow the
/// ownership and borrowing rules and guarantees.
/// - Make sure the data gotten from inside is a valid T.
/// - Make sure data isn't taken from outside the slice.
pub const unsafe fn take_unchecked<T: Sized>(slice: &[u8], idx: usize) -> T {
    unsafe {
        (slice as *const [u8]).cast::<T>().add(idx).read()
    }
}

/// Takes the value from the specified region.
/// 
/// Note: This does NOT zero out the specified region
/// 
/// # Safety
/// - Make sure the data gotten from inside is a valid T
pub const unsafe fn take_zeroed<T: Sized>(slice: &mut [u8], idx: usize) -> Result<T, idx::IdxError> {
    if let Err(err) = validity(slice, idx, core::mem::size_of::<T>()) {
        return Err(err)
    }
    let take: T = take_unchecked(slice, idx);
    write_zeroes_unchecked(slice, idx, core::mem::size_of::<T>());
    Ok(take)
}

/// Takes the value from the specified region.
/// 
/// Note: This does NOT zero out the specified region
/// 
/// # Safety
/// - Make sure the data gotten from inside is a valid T
/// - Make sure data isn't taken from outside the data structure.
pub const unsafe fn take_zeroed_unchecked<T: Sized>(slice: &mut [u8], idx: usize) -> T {
    let take: T = take_unchecked(slice, idx);
    write_zeroes_unchecked(slice, idx, core::mem::size_of::<T>());
    take
}

/// Takes the value from the specified region and writes a new value in it's palce.
/// 
/// # Safety
/// - Make sure for all the data inside to follow the
/// ownership and borrowing rules and guarantees.
/// - Make sure the data gotten from inside is a valid T
pub const unsafe fn replace<T: Sized>(slice: &mut [u8], idx: usize, value: core::mem::ManuallyDrop<T>) -> Result<T, (core::mem::ManuallyDrop<T>, idx::IdxError)> {
    if let Err(err) = validity(slice, idx, core::mem::size_of::<T>()) {
        return Err((value, err));
    }

    Ok(replace_unchecked(slice, idx, value))
}

/// Takes the value from the specified region and writes a new value in it's palce.
/// 
/// # Safety
/// - Make sure for all the data inside to follow the
/// ownership and borrowing rules and guarantees.
/// - Make sure the data gotten from inside is a valid T
/// - Make sure data isn't taken from outside the data structure.
pub const unsafe fn replace_unchecked<T: Sized>(slice: &mut [u8], idx: usize, value: core::mem::ManuallyDrop<T>) -> T {
    let take = take_unchecked::<T>(slice, idx);
    write_unchecked(slice, idx, value);
    take
}
