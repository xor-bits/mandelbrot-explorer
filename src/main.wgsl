struct VertexInput {
	@location(0) pos: vec2<f32>,
	@location(1) uv: vec2<f32>,
	@location(2) col: vec4<f32>,
};

struct FragmentInput {
	@builtin(position) pos: vec4<f32>,
	@location(1) z: vec2<f32>,
};

struct UniformInput {
    aspect: f32,
};

@group(0)
@binding(0)
var<uniform> ubo: UniformInput;

fn mandelbrot(z0: vec2<f32>, max_iter: i32) -> i32 {
    var z = z0;

    var i = 0;
    while (true) {
        var tmp = z.x * z.x - z.y * z.y + z0.x;
        z.y = 2.0 * z.x * z.y + z0.y;
        z.x = tmp;

        if (z.x * z.x + z.y * z.y >= 4.0 || i >= max_iter) {
            break;
        }

        i++;
    }

    return i;

}

@vertex
fn vs_main(vin: VertexInput) -> FragmentInput {
	var fin: FragmentInput;
	fin.pos = vec4<f32>(vin.pos, 0.0, 1.0);
	fin.z = vin.pos;
	return fin;
}

@fragment
fn fs_main(fin: FragmentInput) -> @location(0) vec4<f32> {
    var i = mandelbrot(fin.z, 128);
    var v = sin(f32(128 - i)) * 0.5 + 0.5;
	return vec4(v, v, v, 1.0);
}
