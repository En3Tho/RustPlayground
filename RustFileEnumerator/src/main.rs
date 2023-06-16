use std::env;
use std::fmt::{Debug, Display, Formatter};
use windows::core::*;
use windows::w;
use windows::Win32::Foundation::{
    GetLastError, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE, MAX_PATH,
};
use windows::Win32::Storage::FileSystem::{
    FindClose, FindFileHandle, FindFirstFileW, FindNextFileW,
    FindVolumeClose, FindVolumeHandle, FILE_ATTRIBUTE_DIRECTORY, WIN32_FIND_DATAW,
};

pub trait HasHandle {
    fn value(&self) -> isize;
}

fn handle_eq<T1, T2>(h1: &T1, h2: &T2) -> bool
where
    T1: HasHandle,
    T2: HasHandle,
{
    h1.value() == h2.value()
}

fn is_invalid_handle(h1: &impl HasHandle) -> bool {
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

impl HasHandle for SafeFindFileHandle {
    fn value(&self) -> isize {
        self.0 .0
    }
}

impl HasHandle for HANDLE {
    fn value(&self) -> isize {
        self.0
    }
}

unsafe fn add_pcwstr_to_path_buffer(add_path: PCWSTR, file_name_buffer: &mut Vec<u16>) -> PCWSTR {
    if file_name_buffer.len() > 0 {
        file_name_buffer.set_len(file_name_buffer.len() - 1); // remove 0 at the end

        let add_path_slice = add_path.as_wide();
        if add_path_slice.len() > 0 {
            file_name_buffer.push('\\' as u16);
            file_name_buffer.extend_from_slice(add_path_slice);
        }
    } else {
        let add_path_slice = add_path.as_wide();
        file_name_buffer.extend_from_slice(add_path_slice);
    }

    file_name_buffer.push('\0' as u16);
    PCWSTR(file_name_buffer.as_ptr())
}

unsafe fn trim_path_buffer(file_name_buffer: &mut Vec<u16>, trim_len: usize) {
    file_name_buffer.set_len(file_name_buffer.len() - trim_len);
    let last_index = file_name_buffer.len() - 1;
    file_name_buffer[last_index] = 0;
}

struct PathBuffer<'a> {
    buffer: &'a mut Vec<u16>,
    trim_len: usize,
}

impl<'a> PathBuffer<'a> {
    unsafe fn from_pwcstr_and_path_buffer(
        add_path: PCWSTR,
        file_name_buffer: &'a mut Vec<u16>,
    ) -> Self {
        let old_len = file_name_buffer.len();
        add_pcwstr_to_path_buffer(add_path, file_name_buffer);
        let new_len = file_name_buffer.len();
        let trim_len = new_len - old_len;

        Self {
            buffer: file_name_buffer,
            trim_len,
        }
    }

    unsafe fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.buffer.as_ptr())
    }
}

impl<'a> Drop for PathBuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            trim_path_buffer(&mut self.buffer, self.trim_len);
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

fn format_byte_count_string(mut count: u64) -> String {
    let buf = ["b", "kb", "mb", "gb"];

    let mut counter = 0;

    while counter < buf.len() && count / 1000 > 0 {
        count = count / 1000;
        counter += 1;
    }

    format!("{}{}", count, buf.get(counter).unwrap())
}

struct EnumerateFilesResult {
    path: String,
    total_file_count: u64,
    total_file_size: u64,
    own_size: u64,
    own_files: u64,
}

impl Display for EnumerateFilesResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Path: {}\n", self.path)?;
        write!(f, "Total file count: {}\n", self.total_file_count)?;
        write!(f, "Total file size: {}\n", format_byte_count_string(self.total_file_size))?;
        write!(f, "Own size: {}\n", format_byte_count_string(self.own_size))?;
        write!(f, "Own files: {}\n", self.own_files)?;
        Ok(())
    }
}

impl Debug for EnumerateFilesResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
            } else {
                false
            }
        }
    }
}

unsafe fn enumerate_files<'a>(
    result_buffer: &'a mut Vec<EnumerateFilesResult>,
    path_buffer: &mut Vec<u16>,
    find_data: &mut WIN32_FIND_DATAW,
) -> Result<&'a EnumerateFilesResult> {
    let base_path = PCWSTR(path_buffer.as_mut_ptr());

    let mut result = EnumerateFilesResult {
        path: base_path.to_string()?,
        total_file_count: 0,
        total_file_size: 0,
        own_size: 0,
        own_files: 0,
    };

    let find_file_handle_result: Result<FindFileHandle>;
    {
        let file_name = PathBuffer::from_pwcstr_and_path_buffer(w!("*"), path_buffer);
        find_file_handle_result = FindFirstFileW(file_name.as_pcwstr(), find_data);
    }

    if should_skip_file(&find_file_handle_result) {
        result_buffer.push(result);
        return Ok(&result_buffer.last().unwrap());
    }

    let find_file_handle = SafeFindFileHandle(find_file_handle_result?);

    if is_invalid_handle(&find_file_handle) {
        check_win32_error()?;
    }

    loop {
        if find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 > 0 {
            if find_data.cFileName[0] != '.' as u16 {
                {
                    let child_file_name = PCWSTR(find_data.cFileName.as_ptr());
                    let child_file_name =
                        PathBuffer::from_pwcstr_and_path_buffer(child_file_name, path_buffer);
                    let child_info_result =
                        enumerate_files(result_buffer, child_file_name.buffer, find_data);
                    match child_info_result {
                        Ok(child_info) => {
                            result.total_file_size += child_info.total_file_size;
                            result.total_file_count += child_info.total_file_count;
                        }
                        Err(e) => {
                            println!("{} error: {}", result.path, e);
                        }
                    }
                }
            }
        } else {
            let file_size = (find_data.nFileSizeHigh as u64) << 32 | find_data.nFileSizeLow as u64;
            result.own_files += 1;
            result.own_size += file_size;
            result.total_file_count += 1;
            result.total_file_size += file_size;
        }

        if !FindNextFileW(find_file_handle.0, find_data).as_bool() {
            break;
        }
    }

    let err = GetLastError();
    if err != ERROR_NO_MORE_FILES {
        check_win32_error()?;
    }

    result_buffer.push(result);

    return Ok(&result_buffer.last().unwrap());
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().collect();
    if args.is_empty() {
        return Err(Error::new(HRESULT(-1), h!("No arguments provided").clone()));
    }

    for result in &args {
        println!("{}", result)
    }

    unsafe {
        let mut find_data = WIN32_FIND_DATAW::default();
        let mut path_buffer = Vec::<u16>::with_capacity(MAX_PATH as usize);
        let mut result_buffer = Vec::<EnumerateFilesResult>::with_capacity(128);

        let main_path = args.get_mut(1).unwrap().trim_end_matches('\\');
        let mut main_path: Vec<u16> = main_path.encode_utf16().collect();
        main_path.push(0);

        let main_path = PCWSTR::from_raw(main_path.as_ptr());
        add_pcwstr_to_path_buffer(main_path, &mut path_buffer);
        enumerate_files(&mut result_buffer, &mut path_buffer, &mut find_data)?;

        result_buffer.sort_by(|a, b| b.total_file_size.cmp(&a.total_file_size));
        for result in result_buffer.iter().take(10) {
            println!("{}", result);
        }
    }
    Ok(())
}
