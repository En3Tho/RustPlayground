use std::cmp::{max, min};
use std::fmt::Debug;
use std::ptr::slice_from_raw_parts;
use std::slice;
use std::usize::MAX;
use windows::core::*;
use windows::Win32::Foundation::{GetLastError, ERROR_NO_MORE_FILES, MAX_PATH, INVALID_HANDLE_VALUE, HANDLE, ERROR_ACCESS_DENIED};
use windows::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_DIRECTORY, FindClose, FindFileHandle, FindFirstFileW, FindFirstVolumeW, FindNextFileW, FindNextVolumeW, FindVolumeClose, FindVolumeHandle, WIN32_FIND_DATAW};
use windows::{w};

pub trait HandleValue {
    fn value(&self) -> isize;
}

fn handle_eq<T1, T2>(h1: &T1, h2: &T2) -> bool
    where T1: HandleValue, T2: HandleValue {
    h1.value() == h2.value()
}

fn is_invalid_handle(h1: &impl HandleValue) -> bool {
    handle_eq(h1, &INVALID_HANDLE_VALUE)
}

pub struct SafeFindFileHandle(FindFileHandle);

impl Drop for SafeFindFileHandle {
    fn drop(&mut self) {
        unsafe {
            FindClose(self.0);
        }
    }
}

impl HandleValue for SafeFindFileHandle {
    fn value(&self) -> isize {
        self.0.0
    }
}

impl HandleValue for HANDLE {
    fn value(&self) -> isize {
        self.0
    }
}

unsafe fn concat_pcwstr(base_path: PCWSTR, add_path: PCWSTR, file_name_buffer: &mut Vec<u16>) -> PCWSTR {
    file_name_buffer.clear();
    let base_path_slice = base_path.as_wide();
    file_name_buffer.extend_from_slice(base_path_slice);
    file_name_buffer.push('\\' as u16);

    let add_path_slice = add_path.as_wide();
    if add_path_slice.len() > 0 {
        file_name_buffer.extend_from_slice(add_path_slice);
    }
    file_name_buffer.push('\0' as u16);
    PCWSTR(file_name_buffer.as_ptr())
}

pub struct PCWSTRVEC {
    raw_data: Vec<u16>,
    str: PCWSTR
}

impl PCWSTRVEC {
    fn from_vec(vec: Vec<u16>) -> PCWSTRVEC {
        let ptr = vec.as_ptr();
        PCWSTRVEC {
            raw_data: vec,
            str: PCWSTR(ptr)
        }
    }

    unsafe fn from_path_concat(base_path: PCWSTR, add_path: PCWSTR) -> PCWSTRVEC {
        let base_path_slice = base_path.as_wide();
        let add_path_slice = add_path.as_wide();

        let mut vec = Vec::<u16>::with_capacity(base_path_slice.len() + add_path_slice.len() + 2);
        let result = concat_pcwstr(base_path, add_path, &mut vec);
        PCWSTRVEC {
            raw_data: vec,
            str: result
        }
    }
}

pub struct SafeFindVolumeHandle(FindVolumeHandle);

impl Drop for SafeFindVolumeHandle {
    fn drop(&mut self) {
        unsafe {
            FindVolumeClose(self.0);
        }
    }
}

pub unsafe fn check_win32_error() -> Result<()> {
    let err = Error::from_win32();
    if err.code() == HRESULT(0) {
        Ok(())
    } else {
        Err(err)
    }
}

pub unsafe fn u16_to_zero_terminated_slice(value: &[u16]) -> &[u16] {
    let zero_index = value
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(value.len());
    &value[..zero_index]
}

pub unsafe fn u16_to_string(value: &[u16]) -> String {
    let zero_terminated = u16_to_zero_terminated_slice(value);
    String::from_utf16_lossy(zero_terminated)
}

unsafe fn print_file_info(find_data: &WIN32_FIND_DATAW) {
    let c_file_name = u16_to_string(&find_data.cFileName);
    dbg!(c_file_name);
    dbg!(find_data.dwReserved0);
    dbg!(find_data.dwReserved1);
    dbg!(find_data.nFileSizeLow);
    dbg!(find_data.nFileSizeHigh);
    println!();
}

unsafe fn enumerate_volumes() -> Result<()> {
    let mut volume_guid = Vec::<u16>::new();
    volume_guid.reserve(8);
    let volume_handle = SafeFindVolumeHandle(FindFirstVolumeW(&mut volume_guid)?);

    loop {
        dbg!(&volume_guid);

        if !FindNextVolumeW(volume_handle.0, &mut volume_guid).as_bool() {
            break;
        }
    }

    Ok(())
}

struct EnumerateFilesResult {
    path: String,
    total_file_count: u64,
    total_file_size: u64,
    own_size: u64,
    own_files: u64,
}

impl Debug for EnumerateFilesResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumerateFilesResult")
            .field("path", &self.path)
            .field("total_file_count", &self.total_file_count)
            .field("total_file_size", &self.total_file_size)
            .field("own_size", &self.own_size)
            .field("own_files", &self.own_files)
            .finish()
    }
}

unsafe fn should_skip_file(file_handle_result: &Result<FindFileHandle>) -> bool {
    match file_handle_result {
        Ok(_) => false,
        Err(e) => {
            // ACCESS DENIED
            if HRESULT(-2147024891).0 == e.code().0 {
                true
            }
            else {
                false
            }
        }
    }
}



unsafe fn enumerate_files(base_path: PCWSTR, find_data: &mut WIN32_FIND_DATAW) -> Result<EnumerateFilesResult> {
    let file_name = PCWSTRVEC::from_path_concat(base_path, w!("*"));

    let mut result = EnumerateFilesResult {
        path: file_name.str.to_string()?,
        total_file_count: 0,
        total_file_size: 0,
        own_size: 0,
        own_files: 0,
    };

    dbg!(&result.path);

    let find_file_handle_result = FindFirstFileW(file_name.str, find_data);
    if should_skip_file(&find_file_handle_result) {
        return Ok(result)
    }

    let find_file_handle = SafeFindFileHandle(find_file_handle_result?);

    if is_invalid_handle(&find_file_handle) {
        check_win32_error()?;
    }

    loop {
        if find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 > 0 {
            if find_data.cFileName[0] != '.' as u16 {
                let child_file_name = PCWSTR(find_data.cFileName.as_ptr());
                let child_path = PCWSTRVEC::from_path_concat(base_path, child_file_name);
                let child_info = enumerate_files(child_path.str, find_data)?;
                result.total_file_size += child_info.total_file_size;
                result.total_file_count += child_info.total_file_count;
            }
        } else {
            let file_size = (find_data.nFileSizeHigh as u64) << 32 | find_data.nFileSizeLow as u64;
            result.own_files += 1;
            result.own_size += file_size;
        }

        if !FindNextFileW(find_file_handle.0, find_data).as_bool() {
            break;
        }
    }

    let err = GetLastError();
    if err != ERROR_NO_MORE_FILES {
        check_win32_error()?;
    }

    dbg!(&result);

    Ok(result)
}

fn main() -> Result<()> {
    unsafe {
        let mut find_data = WIN32_FIND_DATAW::default();
        let result = enumerate_files(w!("C:"), &mut find_data)?;
        dbg!(result);
    }
    Ok(())
}
