#[cfg_attr(
    target_os = "android",
    ndk_glue::main(logger(level = "debug", tag = "MBE"))
)]
pub fn main() {
    mandelbrot_explorer_core::main(true);
}
