mod device_state_change;
mod event_listener;
mod utils;

#[macro_use]
extern crate windows_service;

use std::ffi::OsString;
use windows_service::service::ServiceControl;
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

unsafe fn run_service(_: Vec<OsString>) -> Result<(), windows_service::Error> {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                // Handle stop event and return control back to the system.
                ServiceControlHandlerResult::NoError
            }
            // All services must accept Interrogate even if it's a no-op.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    event_listener::win_main();
    let _status_handle = service_control_handler::register("myservice", event_handler)?;
    Ok(())
}

define_windows_service!(ffi_service_main, my_service_main);
fn my_service_main(arguments: Vec<OsString>) {
    unsafe {
        if let Err(_e) = run_service(arguments) {
            // Handle errors in some way.
        }
    }
}

fn main() -> Result<(), windows_service::Error> {
    // // Register generated `ffi_service_main` with the system and start the service, blocking
    // // this thread until the service is stopped.
    // TODO: windows service
    // service_dispatcher::start("myservice", ffi_service_main)?;

    unsafe {
        event_listener::win_main();
    }

    Ok(())
}
