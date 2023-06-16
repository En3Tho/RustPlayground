use crate::utils::*;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_Get_DevNode_Status, SetupDiChangeState, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
    SetupDiGetDevicePropertyW, SetupDiSetClassInstallParamsW, CM_PROB_DISABLED, CR_SUCCESS,
    DICS_DISABLE, DICS_ENABLE, DICS_FLAG_GLOBAL, DIF_PROPERTYCHANGE, DIGCF_ALLCLASSES, HDEVINFO,
    SP_CLASSINSTALL_HEADER, SP_DEVINFO_DATA, SP_PROPCHANGE_PARAMS,
};
use windows::Win32::Devices::Properties::{DEVPKEY_Device_HardwareIds, DEVPROPTYPE};
use windows::{
    core::*, Devices::Enumeration::*, Foundation::*, Win32::Foundation::*,
    Win32::System::Threading::*,
};

unsafe fn set_device_state(
    dev_info: HDEVINFO,
    dev_data: &mut SP_DEVINFO_DATA,
    state_change: u32,
) -> Result<()> {
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
    SetupDiSetClassInstallParamsW(
        dev_info,
        Some(dev_data),
        Some(sp_header_ptr),
        size_of_u32::<SP_PROPCHANGE_PARAMS>(),
    );
    check_win32_error()?;

    println!("Calling SetupDiChangeState");
    SetupDiChangeState(dev_info, to_mut_ptr(dev_data));
    check_win32_error()?;
    Ok(())
}

unsafe fn get_device_state(dev_data: &SP_DEVINFO_DATA) -> Result<bool> {
    let mut status = 0;
    let mut problem_number = 0;
    let result = CM_Get_DevNode_Status(&mut status, &mut problem_number, dev_data.DevInst, 0);
    if result == CR_SUCCESS {
        return Ok(problem_number != CM_PROB_DISABLED);
    }
    check_win32_error()?;
    Ok(false)
}

unsafe fn get_device_id(
    dev_info: HDEVINFO,
    dev_data: &mut SP_DEVINFO_DATA,
    buf: &mut Vec<u8>,
) -> Result<HSTRING> {
    let mut prop_type = DEVPROPTYPE::default();
    let mut buf_size = u32::default();
    SetupDiGetDevicePropertyW(
        dev_info,
        dev_data,
        &DEVPKEY_Device_HardwareIds,
        &mut prop_type,
        None,
        Some(&mut buf_size),
        0,
    );

    ensure_capacity(buf, buf_size as usize);
    let buf_slice = &mut buf.as_mut_slice()[0..buf_size as usize];
    SetupDiGetDevicePropertyW(
        dev_info,
        dev_data,
        &DEVPKEY_Device_HardwareIds,
        &mut prop_type,
        Some(buf_slice),
        None,
        0,
    );

    HSTRING::from_wide(to_u16_slice(buf_slice))
}

pub unsafe fn change_device_state(turn_on: bool, device_id: &str) -> Result<()> {
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
        if hardware_id
            .to_string()
            .contains(device_id)
        {
            let turned_on = get_device_state(&dev_data)?;
            if turned_on != turn_on {
                if turned_on {
                    println!("Disabling device");
                    set_device_state(dev_info, &mut dev_data, DICS_DISABLE)?;
                } else {
                    println!("Enabling device");
                    set_device_state(dev_info, &mut dev_data, DICS_ENABLE)?;
                }
                if turned_on == get_device_state(&dev_data)? {
                    eprintln!("Failed to change device state");
                }
            }
            break;
        }
    }

    Ok(())
}

pub unsafe fn _file_watcher() -> Result<()> {
    let watcher = DeviceInformation::CreateWatcher()?;
    let handle = SafeHandle(CreateEventW(None, true, false, None)?);

    watcher.Added(&TypedEventHandler::<DeviceWatcher, DeviceInformation>::new(
        |_, info| {
            let info = info.as_ref().expect("info");
            let name = info.Name()?;

            if name.to_string().contains("Intel") {
                println!("Id: {}, Name: {}", info.Id()?, info.Name()?);
            }
            Ok(())
        },
    ))?;

    watcher.EnumerationCompleted(&TypedEventHandler::new(move |_, _| {
        println!("done!");
        SetEvent(handle.0);
        Ok(())
    }))?;

    watcher.Start()?;
    WaitForSingleObject(handle.0, INFINITE);
    Ok(())
}