use shader::{FractalShader, Ubo};
use std::sync::Arc;

use srs2dge::prelude::*;

//

mod shader;

//

struct App {
    target: Target,
    ws: WindowState,

    quad: BatchRenderer,
    ubo: UniformBuffer<Ubo>,
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

    // mvp matrix
    let ubo = UniformBuffer::new_single(
        &target,
        Ubo {
            //mvp: Mat4::IDENTITY,
            aspect: 1.0,
        },
    );

    // custom shader for fractal drawing
    let shader = FractalShader::new(&target);

    // state managers
    let ws = WindowState::new(&window);

    App {
        target,
        ws,

        quad,
        ubo,
        shader,
    }
}

async fn event(app: &mut App, event: Event<'_>, _: &EventLoopTarget, control: &mut ControlFlow) {
    // state manager event handling
    app.ws.event(&event);

    // stop if window should close
    if app.ws.should_close {
        *control = ControlFlow::Exit;
    }
}

async fn draw(app: &mut App) {
    let mut frame = app.target.get_frame();
    let (vbo, ibo, count) = app.quad.generate(&mut app.target, &mut frame);
    let bg = app.shader.bind_group(&app.ubo);

    frame
        .primary_render_pass()
        .bind_vbo(&vbo)
        .bind_ibo(&ibo)
        .bind_shader(&app.shader)
        .bind_group(&bg)
        .draw_indexed(0..count, 0, 0..1);
    app.target.finish_frame(frame);
}

fn main() {
    app!(init, event, draw);
}
