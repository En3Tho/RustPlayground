use crate::device_state_change;
use crate::utils::{size_of_u32, to_u16_bytes};
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcA, DispatchMessageW, GetMessageW, RegisterClassExA, ShowWindow, TranslateMessage, CW_USEDEFAULT, HCURSOR, HICON, HMENU, MSG, PBT_APMRESUMEAUTOMATIC, PBT_APMSUSPEND, SW_HIDE, WINDOW_EX_STYLE, WM_POWERBROADCAST, WNDCLASSEXA, WNDCLASS_STYLES, WS_OVERLAPPEDWINDOW, SW_SHOWMINNOACTIVE};

const DEVICE_ID: &str = "USB\\VID_8087&PID_0A2A&REV_0001";

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_POWERBROADCAST => match wparam.0 as u32 {
            PBT_APMSUSPEND | PBT_APMRESUMEAUTOMATIC => {
                let result = device_state_change::change_device_state(
                    wparam.0 as u32 == PBT_APMRESUMEAUTOMATIC,
                    DEVICE_ID
                );
                return LRESULT(if result.is_err() { 1 } else { 0 });
            }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        },
        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}

pub unsafe fn win_main() {
    let wc = WNDCLASSEXA {
        cbSize: size_of_u32::<WNDCLASSEXA>(),
        style: WNDCLASS_STYLES::default(),
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: HMODULE::default(),
        hIcon: HICON::default(),
        hCursor: HCURSOR::default(),
        hbrBackground: HBRUSH::default(),
        lpszMenuName: PCSTR::null(),
        lpszClassName: PCSTR::from_raw(&b"RustWindowClass\0"[0]),
        hIconSm: HICON::default(),
    };

    RegisterClassExA(&wc);

    println!("RegisterClassExA ... ok");

    let class_name = PCWSTR::from_raw(to_u16_bytes("RustWindowClass").as_ptr());
    let window_name = PCWSTR::from_raw(to_u16_bytes("Rust Window").as_ptr());

    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class_name,
        window_name,
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        HWND::default(),
        HMENU::default(),
        HMODULE::default(),
        None,
    );
    ShowWindow(hwnd, SW_HIDE);

    println!("ShowWindow ... ok");

    let mut msg = MSG::default();

    println!("Listening ...");

    while GetMessageW(&mut msg, hwnd, 0, 0).0 > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}
