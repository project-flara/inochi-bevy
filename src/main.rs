use bevy::{prelude::*, window::WindowId, winit::WinitWindows};
use glutin::context::{ContextApi, ContextAttributesBuilder};
use inochi2d::{
    camera::Inochi2DCamera, core::Inochi2D, puppet::Inochi2DPuppet, scene::Inochi2DScene,
    MONOTONIC_CLOCK,
};
use raw_window_handle::HasRawWindowHandle;

use glutin::prelude::*;

use std::cell::RefCell;
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
fn inochi(world: &mut World) {
    /* Create a new Inochi2D context */
    let ctx = RefCell::new(Inochi2D::new(MONOTONIC_CLOCK, 800, 800));
    /* Create a new Inochi2D puppet from a file */
    let puppet = RefCell::new(Inochi2DPuppet::new(PathBuf::from("./examples/Midori.inx")).unwrap());

    /* Setup the camera and zoom */
    let zoom: f64 = 0.15;
    let cam = RefCell::new(Inochi2DCamera::new(Some(zoom as f32), Some(0.0), Some(0.0)));

    /* Setup the Inochi2D scene to draw */
    let scene = RefCell::new(Inochi2DScene::new());
    world.insert_non_send_resource(Inochi2DRes {
        scene,
        cam,
        puppet,
        ctx,
    });
}

pub fn inochi_render(world: &mut World) {
    let windows = world.get_non_send_resource::<WinitWindows>().unwrap();

    let primary = windows.get_window(WindowId::primary()).unwrap();

    let raw_handle = primary.raw_window_handle();
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version { major: 4, minor: 1})))
        .build(Some(raw_handle));
    
        
    if let Some(inochi) = world.get_non_send_resource::<Inochi2DRes>() {
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
}
