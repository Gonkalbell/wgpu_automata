use wgsl_to_wgpu::{create_shader_module_embedded, MatrixVectorTypes, WriteOptions};

fn main() {
    let name = "automata";
    println!("cargo:rerun-if-changed=src/shaders/{name}.wgsl");
    let wgsl_source = std::fs::read_to_string(format!("src/shaders/{name}.wgsl")).unwrap();

    // Generate the Rust bindings and write to a file.
    let text = create_shader_module_embedded(
        &wgsl_source,
        WriteOptions {
            derive_bytemuck_vertex: true,
            derive_bytemuck_host_shareable: true,
            matrix_vector_types: MatrixVectorTypes::Rust,
            rustfmt: true,
            ..Default::default()
        },
    )
    .unwrap();

    std::fs::write(format!("src/shaders/{name}.rs"), text.as_bytes()).unwrap();
}
