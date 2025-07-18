Writes the given value at the given index.

If you want to store a [?Sized](Sized) value use [write_unsized]

# ERRORS
Will return an error if the write function catches
it'self trying to write in a memory region that is
not assigned to the data structure.

# SAFETY
Make sure for all the data inside to follow the
ownership and borrowing rules and guarantees.