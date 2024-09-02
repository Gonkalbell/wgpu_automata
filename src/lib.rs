#![warn(clippy::all)]

mod app;
pub mod shaders {
    #[allow(warnings)]
    pub mod boids;
}

pub use app::RendererApp;
