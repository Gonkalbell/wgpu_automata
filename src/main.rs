#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui_wgpu::WgpuConfiguration, wgpu};
use std::sync::Arc;

fn get_wgpu_options() -> WgpuConfiguration {
    WgpuConfiguration {
        device_descriptor: Arc::new(|adapter| {
            let base_limits: wgpu::Limits = if adapter.get_info().backend == wgpu::Backend::Gl {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            };
            wgpu::DeviceDescriptor {
                label: Some("egui wgpu device"),
                required_features: wgpu::Features::default()
                    | wgpu::Features::POLYGON_MODE_LINE
                    | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    | wgpu_profiler::GpuProfiler::ALL_WGPU_TIMER_FEATURES,
                required_limits: wgpu::Limits {
                    // When using a depth buffer, we have to be able to create a texture
                    // large enough for the entire surface, and we want to support 4k+ displays.
                    max_texture_dimension_2d: 8192,
                    ..base_limits
                },
                ..Default::default()
            }
        }),
        ..Default::default()
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use wgpu_automata::PUFFIN_GPU_PROFILER;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    puffin::set_scopes_on(true);
    let _cpu_server =
        puffin_http::Server::new(&format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT)).unwrap();
    let _gpu_server = puffin_http::Server::new_custom(
        &format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT + 1),
        |sink| PUFFIN_GPU_PROFILER.lock().unwrap().add_sink(sink),
        |id| _ = PUFFIN_GPU_PROFILER.lock().unwrap().remove_sink(id),
    );

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_icon(
            // NOTE: Adding an icon is optional
            eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                .expect("Failed to load icon"),
        ),
        wgpu_options: get_wgpu_options(),
        ..Default::default()
    };
    eframe::run_native(
        "wgpu automatas",
        native_options,
        Box::new(|cc| Ok(Box::new(wgpu_automata::RendererApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions {
        wgpu_options: get_wgpu_options(),
        ..Default::default()
    };

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(wgpu_automata::RendererApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
