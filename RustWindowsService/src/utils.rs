use std::mem::size_of;
use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::{CloseHandle, HANDLE};

pub fn size_of_u32<T>() -> u32 {
    size_of::<T>() as u32
}

pub unsafe fn to_mut_ptr<T>(value: &mut T) -> *mut T {
    value as *mut T
}

pub unsafe fn check_win32_error() -> windows::core::Result<()> {
    let err = Error::from_win32();
    if err.code() == HRESULT(0) {
        Ok(())
    } else {
        Err(err)
    }
}

pub fn ensure_capacity(buf: &mut Vec<u8>, buf_size: usize) {
    if buf_size > buf.len() {
        let mut new_size = buf.len();
        while new_size < buf_size {
            new_size *= 2;
        }
        *buf = vec![0; new_size];
    }
}

pub fn to_u16_slice(slice: &mut [u8]) -> &mut [u16] {
    let byte_len = slice.len() / 2;
    unsafe { std::slice::from_raw_parts_mut(slice.as_mut_ptr().cast::<u16>(), byte_len) }
}

pub fn to_u16_bytes(value: &str) -> Vec<u16> {
    let mut bytes = value.encode_utf16().collect::<Vec<_>>();
    bytes.push(0);
    bytes
}

pub struct SafeHandle(pub HANDLE);

// implement drop trait using CloseHandle function
impl Drop for SafeHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}