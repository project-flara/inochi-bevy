
use bevy::render::renderer::RenderAdapterInfo;
use bevy::{prelude::*, window::WindowId, winit::WinitWindows};

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;
}

use glutin::api::egl::device::Device;
use glutin::config::{ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder};
use inochi2d::{
    camera::Inochi2DCamera, core::Inochi2D, puppet::Inochi2DPuppet, scene::Inochi2DScene,
    MONOTONIC_CLOCK,
};
use raw_window_handle::HasRawWindowHandle;

use glutin::prelude::*;

use std::cell::RefCell;
use std::ffi::CString;
use std::io::Cursor;
use std::path::PathBuf;
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_startup_system(inochi);
    app.add_system(inochi_render);
    info!("Starting launcher: Native");
    app.run();
}

pub struct Inochi2DRes {
    ctx: RefCell<Inochi2D>,
    puppet: RefCell<Inochi2DPuppet>,
    cam: RefCell<Inochi2DCamera>,
    scene: RefCell<Inochi2DScene>,
}
fn inochi(commands: Commands) {
    use glutin::api::egl::device::Device;
    use glutin::api::egl::display::Display;
    use glutin::config::{ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder};
    use glutin::context::{ContextApi, ContextAttributesBuilder};
    use glutin::prelude::*;

    let devices = Device::query_devices()
        .expect("Failed to query devices")
        .collect::<Vec<_>>();

    for (index, device) in devices.iter().enumerate() {
        println!(
            "Device {}: Name: {} Vendor: {}",
            index,
            device.name().unwrap_or("UNKNOWN"),
            device.vendor().unwrap_or("UNKNOWN")
        );
    }

    let device = devices.first().expect("No available devices");

    // Create a display using the device.
    let display = unsafe { Display::with_device(device, None) }.expect("Failed to create display");

    let template = config_template();
    let config = unsafe { display.find_configs(template) }
        .unwrap()
        .reduce(|config, acc| {
            if config.num_samples() > acc.num_samples() {
                config
            } else {
                acc
            }
        })
        .expect("No available configs");

    println!("Picked a config with {} samples", config.num_samples());

    // Context creation.
    //
    // In particular, since we are doing offscreen rendering we have no raw window
    // handle to provide.
    let context_attributes = ContextAttributesBuilder::new().build(None);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(None);

    let not_current = unsafe {
        display
            .create_context(&config, &context_attributes)
            .unwrap_or_else(|_| {
                display
                    .create_context(&config, &fallback_context_attributes)
                    .expect("failed to create context")
            })
    };

    // Make the context current for rendering
    let _context = not_current.make_current_surfaceless().unwrap();

    // Create a framebuffer for offscreen rendering since we do not have a window.
    let mut framebuffer = 0;
    let mut renderbuffer = 0;
    let gl = gl::Gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        display.get_proc_address(symbol.as_c_str()).cast()
    });
    /* Create a new Inochi2D context */
    let ctx = RefCell::new(Inochi2D::new(MONOTONIC_CLOCK, 800, 800));
    /* Create a new Inochi2D puppet from a file */
    let puppet = RefCell::new(Inochi2DPuppet::new(PathBuf::from("./examples/Midori.inx")).unwrap());

    unsafe {
        gl.GenFramebuffers(1, &mut framebuffer);
        gl.GenRenderbuffers(1, &mut renderbuffer);
        gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        gl.BindRenderbuffer(gl::RENDERBUFFER, renderbuffer);
        gl.RenderbufferStorage(gl::RENDERBUFFER, gl::RGBA, 1280, 720);
        gl.FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::RENDERBUFFER,
            renderbuffer,
        );
    }

  
    /* Setup the camera and zoom */
    let zoom: f64 = 0.15;
    let cam = RefCell::new(Inochi2DCamera::new(Some(zoom as f32), Some(0.0), Some(0.0)));

    /* Setup the Inochi2D scene to draw */
    let scene = RefCell::new(Inochi2DScene::new());
}

pub fn inochi_render(
    inochi: NonSend<Inochi2DRes>,
    adapter: Res<RenderAdapterInfo>,
    windows: NonSend<WinitWindows>,
) {
    let (mut puppet, mut scene, mut ctx) = {
        (
            inochi.puppet.borrow_mut(),
            inochi.scene.borrow_mut(),
            inochi.ctx.borrow_mut(),
        )
    };
    /* Update and then draw the puppet */
    puppet.update();
    puppet.draw();
    /* Draw the scene */

    scene.draw(
        0.0,
        0.0,
        (ctx.view_width + 0) as f32,
        (&ctx.view_height + 0) as f32,
    );
}

fn config_template() -> ConfigTemplate {
    ConfigTemplateBuilder::default()
        .with_alpha_size(8)
        // Offscreen rendering has no support window surface support.
        .with_surface_type(ConfigSurfaceTypes::empty())
        .build()
}
