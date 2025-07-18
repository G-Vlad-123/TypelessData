Writes the given value at the given index.

If you want to store a [?Sized](Sized) value use [write_unsized_unchecked]

# SAFETY
- Make sure for all the data inside to follow the
ownership and borrowing rules and guarantees.
- Make sure no data is written to a region outside of the specified data structure.