
#![allow(unused_imports)]

use crate::*;
use array::DataArray;
use slice::DataSlice;
use boxed::DataBoxed;
use core::mem::ManuallyDrop;

extern crate std;

#[test]
fn print_bytes() {
    let data: DataArray<32> = DataArray {
        inner: core::array::from_fn(|idx| (idx + 1) as u8)
    };

    std::println!("{:?}", data);
    std::println!("\n--------------------\n");
    std::println!("{:4?}", data);
    std::println!("\n--------------------\n");
    std::println!("{:5?}", data);
    std::println!("\n--------------------\n");
    std::println!("{:>5?}", data);
    std::println!("\n--------------------\n");
    std::println!("{:^5?}", data);
    std::println!("\n--------------------\n");
    std::println!("{:<5?}", data);

    panic!()
}

#[test]
fn ownership() {
    let mut data: DataArray<32> = DataArray::uninit();

    struct NoClone(#[allow(unused)] i32);

    let value = NoClone(15);

    unsafe {
        let _ = data.write_unsized(2, &ManuallyDrop::new(value));
    }

    // let moved = value;
}
