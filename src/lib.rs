#![warn(clippy::all)]

mod app;
pub mod shaders {
    #[allow(warnings)]
    pub mod boids;
}

pub use app::RendererApp;

pub static PUFFIN_GPU_PROFILER: std::sync::LazyLock<std::sync::Mutex<puffin::GlobalProfiler>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(puffin::GlobalProfiler::default()));
