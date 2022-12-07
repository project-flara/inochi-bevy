use bevy::prelude::*;
use bevy::window::WindowResized;
use glutin::prelude::PossiblyCurrentGlContext;
use replace_with::replace_with;

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

use glutin::api::egl::device::Device;
use glutin::config::{ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder};

use glutin::api::egl::context::NotCurrentContext;
use inochi2d::{
    camera::Inochi2DCamera, core::Inochi2D, puppet::Inochi2DPuppet, scene::Inochi2DScene,
    MONOTONIC_CLOCK,
};

use std::ffi::CString;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Resource)]
pub struct Inochi2DRes {
    ctx: Mutex<Inochi2D>,
    puppet: Mutex<Inochi2DPuppet>,
    cam: Mutex<Inochi2DCamera>,
    scene: Mutex<Inochi2DScene>,
    gl_ctx: Mutex<NotCurrentContext>,
    renderbuffer: Mutex<u32>,
    framebuffer: Mutex<u32>,
    display: Mutex<glutin::api::egl::display::Display>,
    gl: Mutex<gl::Gl>,
}

fn config_template() -> ConfigTemplate {
    ConfigTemplateBuilder::default()
        .with_alpha_size(8)
        // Offscreen rendering has no support window surface support.
        .with_surface_type(ConfigSurfaceTypes::empty())
        .build()
}

pub struct Inochi2DPlugin;

impl Plugin for Inochi2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::startup)
            .add_system(Self::render)
            .add_system(Self::resize);
    }
}

impl Inochi2DPlugin {
    fn startup(mut commands: Commands) {
        use glutin::api::egl::display::Display;

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
        let display =
            unsafe { Display::with_device(device, None) }.expect("Failed to create display");

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
        let context = not_current.make_current_surfaceless().unwrap();

        // Create a framebuffer for offscreen rendering since we do not have a window.
        let mut framebuffer = 0;
        let mut renderbuffer = 0;

        let gl = gl::Gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            display.get_proc_address(symbol.as_c_str()).cast()
        });
        /* Create a new Inochi2D context */
        let ctx = Mutex::new(Inochi2D::new(MONOTONIC_CLOCK, 800, 800));
        /* Create a new Inochi2D puppet from a file */
        let puppet =
            Mutex::new(Inochi2DPuppet::new(PathBuf::from("./examples/Midori.inx")).unwrap());
        println!("Hii");
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
        println!("Hii");
        /* Setup the camera and zoom */
        let zoom: f64 = 0.15;
        let cam = Mutex::new(Inochi2DCamera::new(Some(zoom as f32), Some(0.0), Some(0.0)));
        println!("Hii");
        /* Setup the Inochi2D scene to draw */
        let scene = Mutex::new(Inochi2DScene::new());
        println!("Hii");
        commands.insert_resource(Inochi2DRes {
            scene,
            cam,
            puppet,
            ctx,
            gl_ctx: Mutex::new(context.make_not_current().unwrap()),
            renderbuffer: Mutex::new(renderbuffer),
            framebuffer: Mutex::new(framebuffer),
            display: Mutex::new(display),
            gl: Mutex::new(gl),
        });
        println!("Hii end");
    }

    fn render(inochi: Res<Inochi2DRes>) {
        let (mut puppet, mut scene, ctx, mut gl_ctx) = {
            (
                inochi.puppet.lock().unwrap(),
                inochi.scene.lock().unwrap(),
                inochi.ctx.lock().unwrap(),
                inochi.gl_ctx.lock().unwrap(),
            )
        };
        replace_with(
            &mut (*gl_ctx),
            || todo!(),
            |gl_ctx| {
                let gl_ctx = gl_ctx.make_current_surfaceless().unwrap();
                /* Update and then draw the puppet */
                puppet.update();
                puppet.draw();
                scene.draw(
                    0.0,
                    0.0,
                    (ctx.view_width + 0) as f32,
                    (&ctx.view_height + 0) as f32,
                );
                println!("Hii end render");

                gl_ctx.make_not_current().unwrap()
            },
        );

        println!("Hi di");
    }

    fn resize(mut resize: EventReader<WindowResized>, inochi: Res<Inochi2DRes>) {
        for resize in resize.iter() {
            println!("Hii end resize rddender");
            let (mut ctx, gl) = { (inochi.ctx.lock().unwrap(), inochi.gl.lock().unwrap()) };
            let w = resize.width as i32 + 0;
            let h = resize.height as i32 + 0;
            ctx.set_viewport(w, h);
            println!("Hii end resize render");
            unsafe {
                gl.Viewport(0, 0, w, h);
            }
            println!("Hii end resize render");
        }
    }
}
