//! I'm using wgpu_bindgen to generate rust bindings for my shaders. It cleans up a whole lot of boilerplate, so overall it's a net positive. However, I do have some issues I'd like to address with the bindings:
//! Vertex Shaders:
//! - If multiple vertex shaders in the same wgsl file, the the generated `vertex_state` will act as if they all have the same input as the first vertex shader defined in the file
//! - wgpu_bindgen does not recognize loose inputs in vertex shaders. It only recognizes input structs
//! Bind Groups
//! - Two shaders in the same file cannot use different resources that share the same group and binding index. The shaders must be in different wgsl files
//! - If two wgsl files share the same bind group (if they import it from the same module), the a different bind group struct will be generated for each shader

use miette::{IntoDiagnostic, Result};
use wgsl_bindgen::{GlamWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};

// src/build.rs
fn main() -> Result<()> {
    WgslBindgenOptionBuilder::default()
        .workspace_root("src/renderer/shaders")
        .add_entry_point("src/renderer/shaders/skybox.wgsl")
        .serialization_strategy(WgslTypeSerializeStrategy::Bytemuck)
        .type_map(GlamWgslTypeMap)
        .derive_serde(true)
        .output("src/renderer/shaders.rs")
        .build()?
        .generate()
        .into_diagnostic()
}
