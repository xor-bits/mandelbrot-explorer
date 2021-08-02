use gears::{module, pipeline, RGBAOutput, Uniform};

#[cfg(feature = "fp64")]
pub use gears::glam::DVec2 as Vec2;
#[cfg(not(feature = "fp64"))]
pub use gears::glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region {
    pub top_left: Vec2,
    pub bottom_right: Vec2,
}

#[derive(Debug, Uniform, Clone, Copy, PartialEq)]
pub struct UniformRegion {
    pub draw_top_left: Vec2,
    pub draw_bottom_right: Vec2,

    pub select_top_left: Vec2,
    pub select_bottom_right: Vec2,

    pub iterations: u32,
}

impl Default for Region {
    fn default() -> Self {
        Self {
            top_left: Vec2::new(-2.2, -1.4),
            bottom_right: Vec2::new(-2.2 + 2.8, 1.4),
        }
    }
}

impl Default for UniformRegion {
    fn default() -> Self {
        Self {
            draw_top_left: Vec2::new(-2.2, -1.4),
            draw_bottom_right: Vec2::new(-2.2 + 2.8, 1.4),

            select_top_left: Vec2::new(-2.2, -1.4),
            select_bottom_right: Vec2::new(-2.2 + 2.8, 1.4),

            iterations: 512,
        }
    }
}

module! {
    kind = "vert",
    path = "core/res/vert.glsl",
    name = "VERT"
}

#[cfg(feature = "fp64")]
module! {
    kind = "frag",
    path = "core/res/frag.glsl",
    name = "FRAG",
    define = "DEF_VEC2_TYPE=dvec2",
    define = "DEF_FLOAT_TYPE=double"
}
#[cfg(not(feature = "fp64"))]
module! {
    kind = "frag",
    path = "core/res/frag.glsl",
    name = "FRAG",
    define = "DEF_VEC2_TYPE=vec2",
    define = "DEF_FLOAT_TYPE=float"
}

pipeline! {
    "Pipeline"
    () -> RGBAOutput

    mod "VERT" as "vert"
    mod "FRAG" as "frag" where { in UniformRegion as 0 }
}
