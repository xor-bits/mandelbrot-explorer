//! controls:
//! - Select area to zoom in to with cursor and LMB
//! - RMB to return to the last zoom
//! - R to reset
//! - F to fix aspect ratio
//! - Hold shift while selecting area to zoom without aspect ratio constraint
//! - Scroll up/down to change the iteration count by 10
//! - Scroll left/right to change the iteration count by 1
use gears::{
    glam::Vec4, ElementState, EventLoopTarget, Frame, FrameLoopTarget, FramePerfReport,
    ImmediateFrameInfo, KeyboardInput, MouseButton, MouseScrollDelta, RenderRecordBeginInfo,
    RenderRecordInfo, Renderer, RendererRecord, SyncMode, Touch, TouchPhase, UpdateLoopTarget,
    UpdateRecordInfo, VirtualKeyCode, WindowEvent, WriteType,
};
use parking_lot::{RwLock, RwLockWriteGuard};
use shader::{Region, UniformRegion};
use std::{
    mem,
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
    time::Duration,
};
use winit::{dpi::PhysicalPosition, event::Event};

mod shader;

use shader::Vec2;

#[cfg(feature = "fp64")]
type float = f64;
#[cfg(not(feature = "fp64"))]
type float = f32;

fn map<T: Copy + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Mul<Output = T>>(
    value: T,
    low1: T,
    high1: T,
    low2: T,
    high2: T,
) -> T {
    low2 + (value - low1) * (high2 - low2) / (high1 - low1)
}

struct App {
    frame: Frame,
    renderer: Renderer,

    shader: shader::Pipeline,
    settings: RwLock<Settings>,
}

struct Settings {
    draw_region_history: Vec<Region>,
    draw_region: Region,
    select_region: Region,
    iterations: i32,
    dragging: bool,
    shifting: bool,
}

impl App {
    fn new(frame: Frame, renderer: Renderer) -> Arc<RwLock<Self>> {
        let shader = shader::Pipeline::build(&renderer).unwrap();
        let settings = RwLock::new(Settings {
            draw_region_history: Vec::new(),
            draw_region: Region::default(),
            select_region: Region::default(),
            iterations: 512,
            dragging: false,
            shifting: false,
        });

        Arc::new(RwLock::new(Self {
            frame,
            renderer,

            shader,
            settings,
        }))
    }

    fn reset(&self) {
        let mut settings = self.settings.write();
        settings.draw_region_history.clear();
        settings.draw_region = Region::default();
        drop(settings);
        self.fix_aspect();
    }

    fn fix_aspect(&self) {
        let mut settings = self.settings.write();
        let aspect = self.frame.aspect() as float;
        let height = settings.draw_region.bottom_right.y - settings.draw_region.top_left.y;
        settings.draw_region.bottom_right.x = settings.draw_region.top_left.x + height * aspect;
    }

    fn drag_start(&self, settings: &mut RwLockWriteGuard<Settings>) {
        if settings.dragging {
            settings.select_region.bottom_right = settings.select_region.top_left;
        } else {
            let left = settings
                .select_region
                .top_left
                .x
                .min(settings.select_region.bottom_right.x);
            let right = settings
                .select_region
                .top_left
                .x
                .max(settings.select_region.bottom_right.x);
            let top = settings
                .select_region
                .top_left
                .y
                .min(settings.select_region.bottom_right.y);
            let bottom = settings
                .select_region
                .top_left
                .y
                .max(settings.select_region.bottom_right.y);

            let mut old_draw_region = Region {
                top_left: Vec2::new(left, top),
                bottom_right: Vec2::new(right, bottom),
            };
            mem::swap(&mut old_draw_region, &mut settings.draw_region);
            settings.draw_region_history.push(old_draw_region);
        }
    }

    fn moved(&self, settings: &mut RwLockWriteGuard<Settings>, position: &PhysicalPosition<f64>) {
        let cursor = position.to_logical(self.frame.scale());
        let cursor = Vec2::new(
            map(
                cursor.x,
                0.0,
                self.frame.size().0 as float,
                settings.draw_region.top_left.x,
                settings.draw_region.bottom_right.x,
            ),
            map(
                cursor.y,
                0.0,
                self.frame.size().1 as float,
                settings.draw_region.top_left.y,
                settings.draw_region.bottom_right.y,
            ),
        );

        if settings.dragging {
            settings.select_region.bottom_right = cursor;
            if !settings.shifting {
                let aspect = self.frame.aspect() as float;
                let dist = (settings.select_region.top_left - cursor).length()
                    / std::f64::consts::SQRT_2 as float;
                let sign = (cursor - settings.select_region.top_left).signum();

                settings.select_region.bottom_right =
                    settings.select_region.top_left + dist * sign * Vec2::new(aspect, 1.0);
            }
        } else {
            settings.select_region.top_left = cursor;
        }
    }

    fn back(&self) {
        let mut settings = self.settings.write();
        if let Some(last) = settings.draw_region_history.pop() {
            settings.draw_region = last;
        }
    }
}

impl UpdateLoopTarget for App {
    fn update(&self, _: &Duration) {}
}

impl EventLoopTarget for App {
    fn event(&self, event: &WindowEvent) {
        match event {
            WindowEvent::Touch(Touch {
                phase, location, ..
            }) => {
                let mut settings = self.settings.write();
                if *phase == TouchPhase::Started
                    || *phase == TouchPhase::Ended
                    || *phase == TouchPhase::Cancelled
                {
                    self.moved(&mut settings, location);
                    settings.dragging = *phase == TouchPhase::Started;
                    self.drag_start(&mut settings);
                } else {
                    self.moved(&mut settings, location)
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let mut settings = self.settings.write();
                self.moved(&mut settings, position)
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                let mut settings = self.settings.write();
                settings.dragging = *state == ElementState::Pressed;
                self.drag_start(&mut settings);
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::R),
                        ..
                    },
                ..
            } => self.reset(),
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: None,
                        scancode: 0,
                        ..
                    },
                ..
            } => self.back(),
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::LShift),
                        state,
                        ..
                    },
                ..
            } => {
                self.settings.write().shifting = *state == ElementState::Pressed;
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(x, y),
                ..
            } => {
                self.settings.write().iterations += (*x as i32) + (*y as i32) * 10;
            }
            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state: ElementState::Released,
                ..
            } => self.back(),
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::F),
                        ..
                    },
                ..
            } => {
                self.fix_aspect();
            }
            WindowEvent::Resized(_) => self.fix_aspect(),
            _ => {}
        }
    }
}

impl RendererRecord for App {
    fn immediate(&self, imfi: &ImmediateFrameInfo) {
        let settings = self.settings.read();
        let select = if settings.dragging {
            &settings.select_region
        } else {
            &settings.draw_region
        };
        let region = UniformRegion {
            draw_top_left: settings.draw_region.top_left,
            draw_bottom_right: settings.draw_region.bottom_right,

            select_top_left: select.top_left,
            select_bottom_right: select.bottom_right,

            iterations: settings.iterations.max(0) as u32,
        };

        match self.shader.write_fragment_uniform(imfi, &region).unwrap() {
            WriteType::Write => {
                println!();
                println!("|{:<80}|", settings.draw_region.top_left.to_string());
                if settings.draw_region == settings.select_region {
                    println!("|{:^80}|", "");
                } else {
                    println!("|  {:<78}|", settings.select_region.top_left.to_string());
                }
                println!("|{:^80}|", "");
                println!("|{:^80}|", format!("{} iters", settings.iterations));
                println!("|{:^80}|", "");
                if settings.draw_region == settings.select_region {
                    println!("|{:^80}|", "");
                } else {
                    println!(
                        "|{:>78}  |",
                        settings.select_region.bottom_right.to_string()
                    );
                }
                println!("|{:>80}|", settings.draw_region.bottom_right.to_string());
            }
            _ => {}
        }
    }

    unsafe fn update(&self, uri: &UpdateRecordInfo) -> bool {
        self.shader.update(uri)
    }

    fn begin_info(&self) -> RenderRecordBeginInfo {
        RenderRecordBeginInfo {
            clear_color: Vec4::new(1.0, 0.0, 0.5, 1.0),
            debug_calls: false,
        }
    }

    unsafe fn record(&self, rri: &RenderRecordInfo) {
        self.shader.draw(rri).direct(6, 0).execute();
    }
}

impl FrameLoopTarget for App {
    fn frame(&self) -> FramePerfReport {
        self.renderer.frame(self)
    }
}

pub fn main(android: bool) {
    let (frame, event_loop) = Frame::new()
        .with_title("ShaderPG")
        .with_size(600, 600)
        .with_min_size(64, 64)
        .build();

    let mut frame = Some(frame);
    let mut app: Option<Arc<RwLock<App>>> = None;

    let make_app = |frame: &mut Option<Frame>, app: &mut Option<Arc<RwLock<App>>>| {
        let frame = frame.take().unwrap();
        let context = frame.default_context().unwrap();

        let renderer = Renderer::new()
            .with_sync(SyncMode::Immediate)
            .build(context)
            .unwrap();

        *app = Some(App::new(frame, renderer));
    };

    if !android {
        make_app(&mut frame, &mut app);
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => {
                if let Some(app) = app.as_ref() {
                    app.write().event(&event);
                }

                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    _ => (),
                }
            }
            Event::RedrawEventsCleared => {
                if let Some(app) = app.as_ref() {
                    app.write().frame();
                }
            }
            Event::Resumed => {
                if android {
                    make_app(&mut frame, &mut app)
                }
            }
            _ => {}
        }
    });

    /* let _ = UpdateLoop::new()
        .with_rate(UpdateRate::PerSecond(1))
        .with_target(app.clone())
        .build()
        .run();

    FrameLoop::new()
        .with_event_loop(event_loop)
        .with_frame_target(app.clone())
        .with_event_target(app)
        .build()
        .run(); */
}
