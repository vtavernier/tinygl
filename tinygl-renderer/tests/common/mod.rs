#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::JsCast;
    pub use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    pub fn with_window<F: FnOnce(glow::Context, cgmath::Vector2<u32>, &'static str)>(cb: F) {
        let document = web_sys::window()
            .expect("failed to get window")
            .document()
            .expect("failed to get document");

        // Create canvas
        let canvas = document
            .create_element("canvas")
            .expect("failed to create canvas");
        canvas
            .set_attribute("id", "canvas")
            .expect("failed to set canvas id");
        document
            .body()
            .expect("failed to get body")
            .append_child(&canvas)
            .expect("failed to append canvas child");

        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let render_size = cgmath::vec2(canvas.width().into(), canvas.height().into());

        let webgl2_context = canvas
            .get_context("webgl2")
            .expect("failed to get_context(webgl2)")
            .expect("no webgl2 context returned")
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();
        let gl = glow::Context::from_webgl2_context(webgl2_context);
        let shader_version = "#version 300 es";

        cb(gl, render_size, shader_version);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    pub fn with_window<F: FnOnce(glow::Context, cgmath::Vector2<u32>, &'static str)>(cb: F) {
        unsafe {
            env_logger::init();

            let event_loop: glutin::event_loop::EventLoop<()> = {
                #[cfg(target_os = "linux")]
                use glutin::platform::unix::EventLoopExtUnix;
                #[cfg(target_os = "windows")]
                use glutin::platform::unix::EventLoopExtWindows;

                glutin::event_loop::EventLoop::new_any_thread()
            };

            let render_size = cgmath::vec2(1024, 768);
            let size = glutin::dpi::PhysicalSize::new(render_size.x as f64, render_size.y as f64);
            let headless_context = glutin::ContextBuilder::new()
                .build_headless(&event_loop, size)
                .unwrap();
            let headless_context = headless_context.make_current().unwrap();
            let context = glow::Context::from_loader_function(|s| {
                headless_context.get_proc_address(s) as *const _
            });
            let shader_version = "#version 410";

            cb(context, render_size, shader_version);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;
