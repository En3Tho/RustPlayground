use interoptopus::{ffi_function, ffi_type, function, Inventory, InventoryBuilder};
use regex::Regex;
use std::ffi::{c_char, CStr};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref REGEX: Regex = Regex::new(r"\s*((?:XXXX)|-?[0-9]+)\s+(-?[0-9]+)\s+((?:0[xX])?[0-9a-fA-F]+)\s+([0-9a-fA-F]{16})\s+((?:0[xX])?[0-9a-fA-F]+)\s+([^\s].*[^\s])\s+([0-9a-fA-F]{16}:[0-9a-fA-F]{16})\s+([0-9a-fA-F]{16})\s+(-?[0-9]+)\s+([^\s].*[^\s])\s*([^\s].*[^\s])*\s*").unwrap();
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn call_regex(ch: *const c_char) -> bool {
    let c_str = CStr::from_ptr(ch);
    let r_str = c_str.to_str().unwrap();
    REGEX.is_match(r_str)
}

#[ffi_type]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[ffi_function]
#[no_mangle]
pub extern "C" fn my_function(input: Vec2) -> Vec2 {
    Vec2 { y: 10., ..input }
}

pub fn my_inventory() -> Inventory {
    InventoryBuilder::new()
        .register(function!(my_function))
        .register(function!(call_regex))
        .inventory()
}
