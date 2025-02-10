fn main() -> Result<(), windows_service::Error> {
    unsafe {
        event_listener::win_main();
    }

    Ok(())
}
