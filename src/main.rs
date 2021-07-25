//! controls:
//! - Select area to zoom in to with cursor and LMB
//! - RMB to return to the last zoom
//! - R to reset
//! - F to fix aspect ratio
//! - Hold shift while selecting area to zoom without aspect ratio constraint
//! - Scroll up/down to change the iteration count by 10
//! - Scroll left/right to change the iteration count by 1

use gears::{
    glam::{DVec2, Vec4},
    ElementState, EventLoopTarget, Frame, FrameLoop, FrameLoopTarget, FramePerfReport,
    ImmediateFrameInfo, KeyboardInput, MouseButton, MouseScrollDelta, RenderRecordBeginInfo,
    RenderRecordInfo, Renderer, RendererRecord, SyncMode, UpdateLoop, UpdateLoopTarget, UpdateRate,
    UpdateRecordInfo, VirtualKeyCode, WindowEvent, WriteType,
};
use parking_lot::RwLock;
use shader::{Region, UniformRegion};
use std::{sync::Arc, time::Duration};

mod shader;

fn map(value: f64, low1: f64, high1: f64, low2: f64, high2: f64) -> f64 {
    low2 + (value - low1) * (high2 - low2) / (high1 - low1)
}

struct App {
    frame: Frame,
    renderer: Renderer,

    shader: shader::Pipeline,

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

        let app = Self {
            frame,
            renderer,

            shader,

            draw_region_history: Vec::new(),
            draw_region: Region::default(),
            select_region: Region::default(),
            iterations: 512,
            dragging: false,
            shifting: false,
        };

        Arc::new(RwLock::new(app))
    }

    fn reset(&mut self) {
        self.draw_region_history.clear();
        self.draw_region = Region::default();
        self.fix_aspect();
    }

    fn fix_aspect(&mut self) {
        let aspect = self.frame.aspect() as f64;
        let height = self.draw_region.bottom_right.y - self.draw_region.top_left.y;
        self.draw_region.bottom_right.x = self.draw_region.top_left.x + height * aspect;
    }
}

impl UpdateLoopTarget for App {
    fn update(&mut self, _: &Duration) {}
}

impl EventLoopTarget for App {
    fn event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let cursor = position.to_logical(self.frame.scale());
                let cursor = DVec2::new(
                    map(
                        cursor.x,
                        0.0,
                        self.frame.size().0 as f64,
                        self.draw_region.top_left.x,
                        self.draw_region.bottom_right.x,
                    ),
                    map(
                        cursor.y,
                        0.0,
                        self.frame.size().1 as f64,
                        self.draw_region.top_left.y,
                        self.draw_region.bottom_right.y,
                    ),
                );

                if self.dragging {
                    self.select_region.bottom_right = cursor;
                    if !self.shifting {
                        let aspect = self.frame.aspect() as f64;
                        let dist = (self.select_region.top_left - cursor).length()
                            / std::f64::consts::SQRT_2;
                        let sign = (cursor - self.select_region.top_left).signum();

                        self.select_region.bottom_right =
                            self.select_region.top_left + dist * sign * DVec2::new(aspect, 1.0);
                    }
                } else {
                    self.select_region.top_left = cursor;
                }
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.dragging = *state == ElementState::Pressed;
                if self.dragging {
                    self.select_region.bottom_right = self.select_region.top_left;
                } else {
                    let left = self
                        .select_region
                        .top_left
                        .x
                        .min(self.select_region.bottom_right.x);
                    let right = self
                        .select_region
                        .top_left
                        .x
                        .max(self.select_region.bottom_right.x);
                    let top = self
                        .select_region
                        .top_left
                        .y
                        .min(self.select_region.bottom_right.y);
                    let bottom = self
                        .select_region
                        .top_left
                        .y
                        .max(self.select_region.bottom_right.y);

                    self.draw_region_history.push(self.draw_region);
                    self.draw_region = Region {
                        top_left: DVec2::new(left, top),
                        bottom_right: DVec2::new(right, bottom),
                    };
                }
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
                        virtual_keycode: Some(VirtualKeyCode::LShift),
                        state,
                        ..
                    },
                ..
            } => {
                self.shifting = *state == ElementState::Pressed;
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(x, y),
                ..
            } => {
                self.iterations += (*x as i32) + (*y as i32) * 10;
            }
            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state: ElementState::Released,
                ..
            } => {
                if let Some(last) = self.draw_region_history.pop() {
                    self.draw_region = last;
                }
            }
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
        let select = if self.dragging {
            &self.select_region
        } else {
            &self.draw_region
        };
        let region = UniformRegion {
            draw_top_left: self.draw_region.top_left,
            draw_bottom_right: self.draw_region.bottom_right,

            select_top_left: select.top_left,
            select_bottom_right: select.bottom_right,

            iterations: self.iterations.max(0) as u32,
        };

        match self.shader.write_fragment_uniform(imfi, &region).unwrap() {
            WriteType::Write => {
                println!();
                println!("|{:<80}|", self.draw_region.top_left.to_string());
                if self.draw_region == self.select_region {
                    println!("|{:^80}|", "");
                } else {
                    println!("|  {:<78}|", self.select_region.top_left.to_string());
                }
                println!("|{:^80}|", "");
                println!("|{:^80}|", format!("{} iters", self.iterations));
                println!("|{:^80}|", "");
                if self.draw_region == self.select_region {
                    println!("|{:^80}|", "");
                } else {
                    println!("|{:>78}  |", self.select_region.bottom_right.to_string());
                }
                println!("|{:>80}|", self.draw_region.bottom_right.to_string());
            }
            _ => {}
        }
    }

    fn update(&self, uri: &UpdateRecordInfo) -> bool {
        unsafe { self.shader.update(uri) }
    }

    fn begin_info(&self) -> RenderRecordBeginInfo {
        RenderRecordBeginInfo {
            clear_color: Vec4::new(1.0, 0.0, 0.5, 1.0),
            debug_calls: false,
        }
    }

    fn record(&self, rri: &RenderRecordInfo) {
        unsafe {
            self.shader.bind(rri);
            self.shader.draw(6, rri);
        }
    }
}

impl FrameLoopTarget for App {
    fn frame(&self) -> FramePerfReport {
        self.renderer.frame(self)
    }
}

fn main() {
    env_logger::init();

    let (frame, event_loop) = Frame::new()
        .with_title("ShaderPG")
        .with_size(600, 600)
        .with_min_size(64, 64)
        .build();

    let context = frame.default_context().unwrap();

    let renderer = Renderer::new()
        .with_sync(SyncMode::Immediate)
        .build(context)
        .unwrap();

    let app = App::new(frame, renderer);

    let _ = UpdateLoop::new()
        .with_rate(UpdateRate::PerSecond(1))
        .with_target(app.clone())
        .build()
        .run();

    FrameLoop::new()
        .with_event_loop(event_loop)
        .with_frame_target(app.clone())
        .with_event_target(app)
        .build()
        .run();
}
