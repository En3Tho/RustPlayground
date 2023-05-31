use std::mem::size_of;
use windows::{core::*, Devices::Enumeration::*, Foundation::*, Win32::System::Threading::*, Win32::Foundation::*};
use windows::Win32::Devices::DeviceAndDriverInstallation::{CM_Get_DevNode_Status, CM_PROB_DISABLED, CR_SUCCESS, DICS_DISABLE, DICS_ENABLE, DICS_FLAG_GLOBAL, DICS_STOP, DIF_PROPERTIES, DIF_PROPERTYCHANGE, DIGCF_ALLCLASSES, HDEVINFO, SetupDiChangeState, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW, SetupDiGetClassInstallParamsW, SetupDiGetDevicePropertyW, SetupDiSetClassInstallParamsW, SP_CLASSINSTALL_HEADER, SP_DEVINFO_DATA, SP_PROPCHANGE_PARAMS};
use windows::Win32::Devices::Properties::{DEVPKEY_Device_HardwareIds, DEVPROPTYPE};

fn size_of_u32<T>() -> u32 {
    size_of::<T>() as u32
}

unsafe fn to_ptr<T>(value: &T) -> *const T {
    value as *const T
}

unsafe fn to_mut_ptr<T>(value: &mut T) -> *mut T {
    value as *mut T
}

unsafe fn check_win32_error() -> Result<()> {
    let err = Error::from_win32();
    if err.code() == HRESULT(0) {
        Ok(())
    } else {
        Err(err)
    }
}

fn ensure_capacity(buf: &mut Vec<u8>, buf_size: usize) {
    if buf_size > buf.len() {
        let mut new_size = buf.len();
        while new_size < buf_size {
            new_size *= 2;
        }
        *buf = vec![0; new_size as usize];
    }
}

fn to_u16_slice(slice: &mut [u8]) -> &mut [u16] {
    let byte_len = slice.len() / 2;
    unsafe {
        std::slice::from_raw_parts_mut(
            slice.as_mut_ptr().cast::<u16>(),
            byte_len,
        )
    }
}

unsafe fn set_device_state(dev_info: HDEVINFO, dev_data: &mut SP_DEVINFO_DATA, state_change: u32) -> Result<()> {
    let ci_header = SP_CLASSINSTALL_HEADER {
        cbSize: size_of_u32::<SP_CLASSINSTALL_HEADER>(),
        InstallFunction: DIF_PROPERTYCHANGE,
    };

    let mut pc_params = SP_PROPCHANGE_PARAMS {
        ClassInstallHeader: ci_header,
        StateChange: state_change,
        Scope: DICS_FLAG_GLOBAL,
        HwProfile: 0,
    };

    let sp_header_ptr = to_mut_ptr(&mut pc_params).cast::<SP_CLASSINSTALL_HEADER>();

    println!("Calling SetupDiSetClassInstallParamsW");
    SetupDiSetClassInstallParamsW(dev_info, Some(dev_data), Some(sp_header_ptr), size_of_u32::<SP_PROPCHANGE_PARAMS>());
    check_win32_error()?;

    println!("Calling SetupDiChangeState");
    SetupDiChangeState(dev_info, to_mut_ptr(dev_data));
    check_win32_error()?;
    Ok(())
}

unsafe fn get_device_state(dev_info: HDEVINFO, dev_data: &mut SP_DEVINFO_DATA) -> Result<u32> {
    let mut ci_header = SP_CLASSINSTALL_HEADER {
        cbSize: size_of_u32::<SP_CLASSINSTALL_HEADER>(),
        InstallFunction: DIF_PROPERTIES,
    };

    let mut prop_size = 0;

    println!("Calling SetupDiGetClassInstallParamsW");
    SetupDiGetClassInstallParamsW(dev_info, Some(dev_data), Some(&mut ci_header), ci_header.cbSize, Some(&mut prop_size));
    check_win32_error()?;

    println!("State: {}", ci_header.InstallFunction);

    Ok(ci_header.InstallFunction)
}

unsafe fn get_device_state_using_cm_get_devnode_status(dev_data: &SP_DEVINFO_DATA) -> Result<bool> {
    let mut status = 0;
    let mut problem_number = 0;
    let result = CM_Get_DevNode_Status(&mut status, &mut problem_number, dev_data.DevInst, 0);
    if result == CR_SUCCESS {
        return Ok(problem_number == CM_PROB_DISABLED);
    }
    check_win32_error()?;
    Ok(false)
}

unsafe fn get_device_id(dev_info: HDEVINFO, dev_data: &mut SP_DEVINFO_DATA, buf: &mut Vec<u8>) -> Result<HSTRING> {
    let mut prop_type = DEVPROPTYPE::default();
    let mut buf_size = u32::default();
    SetupDiGetDevicePropertyW(dev_info, dev_data, &DEVPKEY_Device_HardwareIds, &mut prop_type, None, Some(&mut buf_size), 0);

    ensure_capacity(buf, buf_size as usize);
    let buf_slice = &mut buf.as_mut_slice()[0..buf_size as usize];
    SetupDiGetDevicePropertyW(dev_info, dev_data, &DEVPKEY_Device_HardwareIds, &mut prop_type, Some(buf_slice), None, 0);

    HSTRING::from_wide(to_u16_slice(buf_slice))
}

pub unsafe fn check_setup_di() -> Result<()> {
    let dev_info = SetupDiGetClassDevsW(None, None, None, DIGCF_ALLCLASSES)?;

    let mut dev_data = SP_DEVINFO_DATA::default();
    dev_data.cbSize = size_of_u32::<SP_DEVINFO_DATA>();

    let mut buf = vec![0; 256 as usize];

    for i in 0..core::u32::MAX {
        let result = SetupDiEnumDeviceInfo(dev_info, i, &mut dev_data);
        if !result.as_bool() {
            if GetLastError() == ERROR_NO_MORE_ITEMS {
                break;
            }
            check_win32_error()?
        }

        let hardware_id = get_device_id(dev_info, &mut dev_data, &mut buf)?;
        if hardware_id.to_string().contains("USB\\VID_8087&PID_0A2A&REV_0001") {
            let device_state = get_device_state_using_cm_get_devnode_status(&dev_data)?;
            dbg!(device_state);
            if device_state {
                println!("Disabling device");
                set_device_state(dev_info, &mut dev_data, DICS_DISABLE)?;
            } else {
                println!("Enabling device");
                set_device_state(dev_info, &mut dev_data, DICS_ENABLE)?;
            }
            break;
        }
    }

    Ok(())
}

pub unsafe fn test() -> Result<()> {
    let watcher = DeviceInformation::CreateWatcher()?;
    let event = CreateEventW(None, true, false, None)?;

    watcher.Added(&TypedEventHandler::<DeviceWatcher, DeviceInformation>::new(
        |_, info| {
            let info = info.as_ref().expect("info");
            let name = info.Name()?;

            // check if hstring "name" has string "intel" inside
            if name.to_string().contains("Intel") {
                println!("Id: {}, Name: {}", info.Id()?, info.Name()?);
            }

            // println!("Id: {}, Name: {}", info.Id()?, info.Name()?);
            Ok(())
        },
    ))?;

    watcher.EnumerationCompleted(&TypedEventHandler::new(move |_, _| {
        println!("done!");
        SetEvent(event);
        Ok(())
    }))?;

    watcher.Start()?;
    WaitForSingleObject(event, INFINITE);
    CloseHandle(event);
    Ok(())
}