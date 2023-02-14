use interoptopus::util::NamespaceMappings;
use interoptopus::{Error, Interop};

#[test]
fn bindings_csharp() -> Result<(), Error> {
    use interoptopus_backend_csharp::overloads::DotNet;
    use interoptopus_backend_csharp::{Config, Generator};

    let config = Config {
        dll_name: "rust_export".to_string(),
        namespace_mappings: NamespaceMappings::new("RustExport"),
        ..Config::default()
    };

    Generator::new(config, rust_export::my_inventory())
        .add_overload_writer(DotNet::new())
        .write_file("G:\\source\\repos\\En3Tho\\RustPlayground\\RustExport\\Interop.cs")?;

    Ok(())
}
