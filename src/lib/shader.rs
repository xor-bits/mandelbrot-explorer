use gears::{glam::DVec2, module, pipeline, RGBAOutput, Uniform};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region {
    pub top_left: DVec2,
    pub bottom_right: DVec2,
}

#[derive(Debug, Uniform, Clone, Copy, PartialEq)]
pub struct UniformRegion {
    pub draw_top_left: DVec2,
    pub draw_bottom_right: DVec2,

    pub select_top_left: DVec2,
    pub select_bottom_right: DVec2,

    pub iterations: u32,
}

impl Default for Region {
    fn default() -> Self {
        Self {
            top_left: DVec2::new(-2.2, -1.4),
            bottom_right: DVec2::new(-2.2 + 2.8, 1.4),
        }
    }
}

impl Default for UniformRegion {
    fn default() -> Self {
        Self {
            draw_top_left: DVec2::new(-2.2, -1.4),
            draw_bottom_right: DVec2::new(-2.2 + 2.8, 1.4),

            select_top_left: DVec2::new(-2.2, -1.4),
            select_bottom_right: DVec2::new(-2.2 + 2.8, 1.4),

            iterations: 512,
        }
    }
}

module! {
    kind = "vert",
    path = "res/vert.glsl",
    name = "VERT"
}

module! {
    kind = "frag",
    path = "res/frag.glsl",
    name = "FRAG"
}

pipeline! {
    "Pipeline"
    () -> RGBAOutput

    mod "VERT" as "vert"
    mod "FRAG" as "frag" where { in UniformRegion as 0 }
}
