fn main() -> Result<(), Box<dyn std::error::Error>> {
    build("client")?;
    build("cluster")?;
    Ok(())
}

fn build(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let proto = format!("../proto/meridian/meridian_{}_v010.proto", name);
    println!("cargo:rerun-if-changed={}", proto);
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("../proto")
        .compile(&[proto.as_str()], &["../proto/meridian"])?;
    Ok(())
}
