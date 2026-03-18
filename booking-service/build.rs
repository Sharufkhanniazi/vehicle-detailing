fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile_well_known_types(true) // optional, useful if you use Timestamp etc.
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]") // <--- add Serde
        .compile(&["../proto/pricing.proto"], &["../proto"])?; // compile proto files

    Ok(())
}