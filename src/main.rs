use gui::{root, Root};
use rug::{ops::CompleteRound, Assign, Float};
use shader::{FractalShader, Ubo, ZoomPointArray};
use srs2dge::winit::event::{MouseScrollDelta, WindowEvent};
use std::{
    ops::{Add, AddAssign, SubAssign},
    sync::Arc,
};

use srs2dge::prelude::*;

//

mod gui;
mod shader;

//

struct App {
    target: Target,
    ws: WindowState,
    ks: KeyboardState,
    ul: Option<UpdateLoop>,

    frames: Reporter,
    dataset: Reporter,

    zoom: f32,
    points_len: u32,
    point: (Float, Float),

    quad: BatchRenderer,
    ubo: UniformBuffer<Ubo>,
    points: UniformBuffer<ZoomPointArray>,
    shader: FractalShader,

    gui: Gui,
    root: Root,
    debug_info: bool,
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
    // let (points, points_len) = points(&point);
    let points = UniformBuffer::new(&target, 1048576); // 1MiB
    let points_len = 0;

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

    // gui
    let gui = Gui::new(&target);
    let root = root();

    // state managers
    let ws = WindowState::new(&window);
    let ks = KeyboardState::new();
    let ul = Some(UpdateLoop::new(UpdateRate::PerSecond(60)));

    App {
        target,
        ws,
        ks,
        ul,

        frames: Reporter::new(),
        dataset: Reporter::new(),

        zoom: 1.0,
        points_len,
        point,

        quad,
        ubo,
        points,
        shader,

        gui,
        root,
        debug_info: false,
    }
}

async fn event(app: &mut App, event: Event<'_>, _: &EventLoopTarget, control: &mut ControlFlow) {
    // state manager event handling
    app.ws.event(&event);
    app.ks.event(&event);

    // app events
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

    // gui events
    if let Some(e) = event.to_static() {
        app.gui.event(&mut app.root, e);
    }

    // stop if window should close
    if app.ws.should_close {
        *control = ControlFlow::Exit;
    }
}

fn update(app: &mut App, frame: &mut Frame) {
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
    if app.ks.just_pressed(VirtualKeyCode::F1) {
        app.debug_info = !app.debug_info;
    }
    app.ks.clear();

    if app.debug_info {
        let format_float = |float: &Float| {
            itertools::intersperse(
                float
                    .to_string()
                    .split('e')
                    .map(|s| s.trim_end_matches('0').trim_end_matches('.')),
                "e",
            )
            .collect::<String>()
            // let (sign, string, exp) = float.to_sign_string_exp(10, None);
            // format!(
            //     "{}{}{}",
            //     if sign { "-" } else { "" },
            //     string.trim_end_matches('0').trim_end_matches('.'),
            //     if let Some(exp) = exp {
            //         format!("e{exp}")
            //     } else {
            //         String::default()
            //     }
            // )
        };

        app.root.set_text(format!(
            r#"- [f1] -
Iterations: {}
Real: {:2.}
Imag: {:2.}
Zoom: {:2.}
{}
{}"#,
            app.points_len,
            format_float(&app.point.0),
            format_float(&app.point.1),
            app.zoom.exp(),
            app.frames
                .last()
                .map(|(int, ps)| format!("FPS: {ps} ({int:?}/f)"))
                .unwrap_or_default(),
            app.dataset
                .last()
                .map(|(int, _)| format!("Dataset took: {int:?} ({})", app.points_len))
                .unwrap_or_default(),
        ));
    } else {
        app.root.set_text("- [f1] -");
    };

    if updated {
        // deep zoom point buffer
        let timer = app.dataset.begin();
        let (points, len) = points(&app.point);
        app.points
            .upload_iter(&mut app.target, frame, 0, len as u64, points.iter());
        app.points_len = len;
        app.dataset.end(timer);
    }
}

fn points(point: &(Float, Float)) -> (Vec<Vec2>, u32) {
    let points = deep_zoom(point, 2048).collect::<Vec<_>>();
    let points_len = points.len() as u32;

    (points, points_len)
}

async fn draw(app: &mut App) {
    let mut frame = app.target.get_frame();

    // fixed timestep updates
    let mut ul = app.ul.take().unwrap();
    ul.update(|| {
        update(app, &mut frame);
    });
    app.ul = Some(ul);

    let timer = app.frames.begin();

    // update uniform buffers
    app.ubo.upload_single(
        &mut app.target,
        &mut frame,
        &Ubo {
            aspect: app.ws.aspect,
            zoom: app.zoom.exp(),
            points: app.points_len,
        },
    );

    // generate main quad
    let (vbo, ibo, count) = app.quad.generate(&mut app.target, &mut frame);
    let bg = app.shader.bind_group((&app.ubo, &app.points));

    // gui
    let gui = app.gui.draw(&mut app.root, &mut app.target, &mut frame);

    // render commands
    frame
        .primary_render_pass()
        .bind_vbo(&vbo)
        .bind_ibo(&ibo)
        .bind_shader(&app.shader)
        .bind_group(&bg)
        .draw_indexed(0..count, 0, 0..1)
        .draw_gui(&gui);

    app.target.finish_frame(frame);
    app.frames.end(timer);
}

fn deep_zoom((real0, imag0): &(Float, Float), iterations: u32) -> impl Iterator<Item = Vec2> + '_ {
    let mut real = real0.clone();
    let mut imag = imag0.clone();

    let mut re = Float::new(512);
    let mut im = Float::new(512);

    const LIMIT: f32 = 1024.0 * 4.0;

    (0..iterations)
        .map(move |_| {
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
}

fn main() {
    app!(init, event, draw);
}
