use rug::{ops::CompleteRound, Assign, Float};
use shader::{FractalShader, Ubo};
use srs2dge::winit::event::{MouseScrollDelta, WindowEvent};
use std::{
    ops::{Add, AddAssign, SubAssign},
    sync::Arc,
};

use srs2dge::prelude::*;

//

mod shader;

//

struct App {
    target: Target,
    ws: WindowState,
    ks: KeyboardState,
    ul: Option<UpdateLoop>,

    zoom: f32,
    points_len: u32,
    point: (Float, Float),

    quad: BatchRenderer,
    ubo: UniformBuffer<Ubo>,
    points: UniformBuffer<[Vec2; 2048]>,
    shader: FractalShader,
}

//

async fn init(target: &EventLoopTarget) -> App {
    // open a window
    let window = WindowBuilder::new()
        .with_visible(false)
        .build(target)
        .expect("Failed to open a window");
    let window = Arc::new(window);

    // init the render engine
    let engine = Engine::new();
    // and create a target that uses the window
    let target = engine.new_target(window.clone()).await;

    // a quad covering the whole screen
    let mut quad = BatchRenderer::new(&target);
    quad.push_with(QuadMesh::new_top_left(
        Vec2::new(-1.0, -1.0),
        Vec2::new(2.0, 2.0),
        Color::WHITE,
        Default::default(),
    ));

    let point = (Float::new(512).add(-1.0), Float::new(512).add(0.2));

    // deep zoom point buffer
    let mut points_data = deep_zoom(&point, 2048);
    let points_len = points_data.len() as u32;
    points_data.resize(2048, Vec2::ZERO);
    let mut points = [Vec2::ZERO; 2048];
    points.clone_from_slice(&points_data[..]);
    let points = UniformBuffer::new_single(&target, points);

    // mvp matrix
    let ubo = UniformBuffer::new_single(
        &target,
        Ubo {
            //mvp: Mat4::IDENTITY,
            aspect: 1.0,
            zoom: 1.0,
            points: points_len,
        },
    );

    // custom shader for fractal drawing
    let shader = FractalShader::new(&target);

    // state managers
    let ws = WindowState::new(&window);
    let ks = KeyboardState::new();
    let ul = Some(UpdateLoop::new(UpdateRate::PerSecond(60)));

    App {
        target,
        ws,
        ks,
        ul,

        zoom: 1.0,
        points_len,
        point,

        quad,
        ubo,
        points,
        shader,
    }
}

async fn event(app: &mut App, event: Event<'_>, _: &EventLoopTarget, control: &mut ControlFlow) {
    // state manager event handling
    app.ws.event(&event);
    app.ks.event(&event);

    if let Event::WindowEvent {
        event:
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(x, y),
                ..
            },
        ..
    } = event
    {
        app.zoom -= x * 0.1;
        app.zoom -= y * 0.1;
    }

    // stop if window should close
    if app.ws.should_close {
        *control = ControlFlow::Exit;
    }
}

fn update(app: &mut App) {
    const SPEED: f32 = 0.05;
    let mut updated = false;
    if app.ks.pressed(VirtualKeyCode::Q) {
        app.zoom += 0.1;
    }
    if app.ks.pressed(VirtualKeyCode::E) {
        app.zoom -= 0.1;
    }
    if app.ks.pressed(VirtualKeyCode::A) {
        app.point.0.sub_assign(app.zoom.exp() * SPEED);
        updated = true;
    }
    if app.ks.pressed(VirtualKeyCode::D) {
        app.point.0.add_assign(app.zoom.exp() * SPEED);
        updated = true;
    }
    if app.ks.pressed(VirtualKeyCode::W) {
        app.point.1.add_assign(app.zoom.exp() * SPEED);
        updated = true;
    }
    if app.ks.pressed(VirtualKeyCode::S) {
        app.point.1.sub_assign(app.zoom.exp() * SPEED);
        updated = true;
    }

    if updated {
        upload(app);
    }
}

fn upload(app: &mut App) {
    // deep zoom point buffer
    let mut points_data = deep_zoom(&app.point, 2048);
    let points_len = points_data.len() as u32;
    points_data.resize(2048, Vec2::ZERO);
    let mut points = [Vec2::ZERO; 2048];
    points.clone_from_slice(&points_data[..]);
    let points = UniformBuffer::new_single(&app.target, points);
    app.points = points;
    app.points_len = points_len;
}

async fn draw(app: &mut App) {
    let mut frame = app.target.get_frame();

    let mut ul = app.ul.take().unwrap();
    ul.update(|| {
        update(app);
    });
    app.ul = Some(ul);

    app.ubo.upload_single(
        &mut app.target,
        &mut frame,
        &Ubo {
            aspect: app.ws.aspect,
            zoom: app.zoom.exp(),
            points: app.points_len,
        },
    );

    let (vbo, ibo, count) = app.quad.generate(&mut app.target, &mut frame);
    let bg = app.shader.bind_group((&app.ubo, &app.points));

    frame
        .primary_render_pass()
        .bind_vbo(&vbo)
        .bind_ibo(&ibo)
        .bind_shader(&app.shader)
        .bind_group(&bg)
        .draw_indexed(0..count, 0, 0..1);

    app.target.finish_frame(frame);
}

fn deep_zoom((real0, imag0): &(Float, Float), iterations: u32) -> Vec<Vec2> {
    let mut real = real0.clone();
    let mut imag = imag0.clone();

    let mut re = Float::new(512);
    let mut im = Float::new(512);

    const LIMIT: f32 = 1024.0 * 32.0;

    (0..iterations)
        .map(|_| {
            re.assign(&real * 2.0);
            im.assign(&imag * 2.0);

            // r * r - i * i + r0
            real = (&real * &real - &imag * &imag).complete(512);
            real.add_assign(real0);
            // 2 * r * i + i0
            imag = (&re * &imag + imag0).complete(512);

            (re.to_f32(), im.to_f32())
        })
        .take_while(|(re, im)| (-LIMIT..LIMIT).contains(re) && (-LIMIT..LIMIT).contains(im))
        .map(|(re, im)| Vec2::new(re, im))
        .collect()
}

fn main() {
    app!(init, event, draw);
}
