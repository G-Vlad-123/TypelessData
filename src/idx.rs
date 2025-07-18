
/*!
This module provides helper items for indexing operations on all the data structures.
 */

#[cfg(feature = "new_range_api")]
use core::range;
use core::ops::{
    self,
    Bound
};

/// 
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdxError {
    #[allow(missing_docs)] pub idx: usize,
    #[allow(missing_docs)] pub data_size: usize,
    #[allow(missing_docs)] pub type_size: usize,
}

impl core::error::Error for IdxError {}
impl core::fmt::Display for IdxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.idx > self.data_size {
            write!(f, "Can not acces data at idx `{idx}` because it is greater then `{size}`.", idx = self.idx, size = self.data_size)
        } else if self.idx.checked_add(self.type_size).map(|idx| idx > self.data_size).unwrap_or(true) {
            write!(
                f,
                "Can not acces data at idx `{idx}` because the size of the data is too large and gets out of the memory given to data.",
                idx = self.idx,
            )
        } else {
            unimplemented!("This error should have never been cosntructed and given.")
        }
    }
}

trait Sealed {}
/// A custom index trait.
/// 
/// This trait marks all possible indexing types for a slice,
/// 
/// Reasons why a custom trait is used over anything from [core]:
/// - due to [usize] giving a single item instead of a slice, using the
/// in-built [SliceIndex](core::slice::SliceIndex) trait is not dooable easely, plus plenty of functions
/// take in eather only a usize or a range.
/// - core's [SliceIndex](core::slice::SliceIndex) is already sealed, so using this trait should always work
#[allow(private_bounds)]
pub trait Idx: Sealed {
    /// Gets the starting bound.
    fn start(&self) -> Bound<usize>;
    /// Gets the ending bound.
    fn end(&self) -> Bound<usize>;

    /// Gets the full range.
    #[inline]
    fn range(&self) -> (Bound<usize>, Bound<usize>) {
        (self.start(), self.end())
    }
}

impl<T: Idx> Sealed for &T {}
impl<T: Idx> Idx for &T {
    #[inline] fn start(&self) -> Bound<usize> { (**self).start() }
    #[inline] fn end(&self) -> Bound<usize> { (**self).end() }
    #[inline] fn range(&self) -> (Bound<usize>, Bound<usize>) { (**self).range() }
}

impl<T: Idx> Sealed for &mut T {}
impl<T: Idx> Idx for &mut T {
    #[inline] fn start(&self) -> Bound<usize> { (**self).start() }
    #[inline] fn end(&self) -> Bound<usize> { (**self).end() }
    #[inline] fn range(&self) -> (Bound<usize>, Bound<usize>) { (**self).range() }
}

impl Sealed for usize {}
impl Idx for usize {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Included(*self)
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Included(*self)
    }
}

trait BoundTrait: Copy {
    fn bound(self) -> Bound<usize>;
}

impl BoundTrait for Bound<usize>   { #[inline(always)] fn bound(self) -> Bound<usize> { self } }
impl BoundTrait for Bound<&usize>  { #[inline(always)] fn bound(self) -> Bound<usize> { self.cloned() } }
impl BoundTrait for &Bound<usize>  { #[inline(always)] fn bound(self) -> Bound<usize> { self.clone() } }
impl BoundTrait for &Bound<&usize> { #[inline(always)] fn bound(self) -> Bound<usize> { self.cloned() } }

impl<B1: BoundTrait, B2: BoundTrait> Sealed for (B1, B2) {}
impl<B1: BoundTrait, B2: BoundTrait> Idx for (B1, B2) {
    #[inline] fn start(&self) -> Bound<usize> { self.0.bound() }
    #[inline] fn end(&self) -> Bound<usize> { self.1.bound() }
}

impl Sealed for ops::Range<usize> {}
impl Idx for ops::Range<usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Included(self.start)
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Excluded(self.end)
    }
}

impl Sealed for ops::Range<&usize> {}
impl Idx for ops::Range<&usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Included(*self.start)
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Excluded(*self.end)
    }
}

impl Sealed for ops::RangeInclusive<usize> {}
impl Idx for ops::RangeInclusive<usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Included(*self.start())
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Included(*self.end())
    }
}

impl Sealed for ops::RangeInclusive<&usize> {}
impl Idx for ops::RangeInclusive<&usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Included(**self.start())
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Included(**self.end())
    }
}

impl Sealed for ops::RangeFrom<usize> {}
impl Idx for ops::RangeFrom<usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Included(self.start)
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Unbounded
    }
}

impl Sealed for ops::RangeFrom<&usize> {}
impl Idx for ops::RangeFrom<&usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Included(*self.start)
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Unbounded
    }
}

impl Sealed for ops::RangeTo<usize> {}
impl Idx for ops::RangeTo<usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Unbounded
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Excluded(self.end)
    }
}

impl Sealed for ops::RangeTo<&usize> {}
impl Idx for ops::RangeTo<&usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Unbounded
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Excluded(*self.end)
    }
}

impl Sealed for ops::RangeToInclusive<usize> {}
impl Idx for ops::RangeToInclusive<usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Unbounded
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Included(self.end)
    }
}

impl Sealed for ops::RangeToInclusive<&usize> {}
impl Idx for ops::RangeToInclusive<&usize> {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Unbounded
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Included(*self.end)
    }
}

impl Sealed for ops::RangeFull {}
impl Idx for ops::RangeFull {
    #[inline] fn start(&self) -> Bound<usize> {
        Bound::Unbounded
    }

    #[inline] fn end(&self) -> Bound<usize> {
        Bound::Unbounded
    }
}

#[cfg(feature = "new_range_api")]
mod range_impl {
    use super::*;

    impl Sealed for range::Range<usize> {}
    impl Idx for range::Range<usize> {
        #[inline] fn start(&self) -> Bound<usize> {
            Bound::Included(self.start)
        }
    
        #[inline] fn end(&self) -> Bound<usize> {
            Bound::Excluded(self.end)
        }
    }
    
    impl Sealed for range::Range<&usize> {}
    impl Idx for range::Range<&usize> {
        #[inline] fn start(&self) -> Bound<usize> {
            Bound::Included(*self.start)
        }
    
        #[inline] fn end(&self) -> Bound<usize> {
            Bound::Excluded(*self.end)
        }
    }
    
    impl Sealed for range::RangeInclusive<usize> {}
    impl Idx for range::RangeInclusive<usize> {
        #[inline] fn start(&self) -> Bound<usize> {
            Bound::Included(self.start)
        }
    
        #[inline] fn end(&self) -> Bound<usize> {
            Bound::Included(self.end)
        }
    }
    
    impl Sealed for range::RangeInclusive<&usize> {}
    impl Idx for range::RangeInclusive<&usize> {
        #[inline] fn start(&self) -> Bound<usize> {
            Bound::Included(*self.start)
        }
    
        #[inline] fn end(&self) -> Bound<usize> {
            Bound::Included(*self.end)
        }
    }
    
    impl Sealed for range::RangeFrom<usize> {}
    impl Idx for range::RangeFrom<usize> {
        #[inline] fn start(&self) -> Bound<usize> {
            Bound::Included(self.start)
        }
    
        #[inline] fn end(&self) -> Bound<usize> {
            Bound::Unbounded
        }
    }
    
    impl Sealed for range::RangeFrom<&usize> {}
    impl Idx for range::RangeFrom<&usize> {
        #[inline] fn start(&self) -> Bound<usize> {
            Bound::Included(*self.start)
        }
    
        #[inline] fn end(&self) -> Bound<usize> {
            Bound::Unbounded
        }
    }
}
