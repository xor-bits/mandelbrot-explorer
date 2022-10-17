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
    zoom: f32,
    points: u32,
};

@group(0)
@binding(0)
var<uniform> ubo: UniformInput;

@group(0)
@binding(1)
var<uniform> zoom: array<vec4<f32>, 1024>;

//

fn get_zoom_point(i: u32) -> vec2<f32> {
    var sub_i = i / 2u;
    var i = i % 2u;

    if (i == 0u) {
        return zoom[sub_i].xy;
    } else {
        return zoom[sub_i].zw;
    }
}

// basic algo
fn mandelbrot(z0: vec2<f32>, max_iter: u32) -> u32 {
    var z = z0;

    var i = 0u;
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

// deep zoom algo
fn mandelbrot_2(z0: vec2<f32>) -> u32 {
    var z = z0;

    var i: u32;
    for (i = 0u; i < ubo.points; i++) {
        var dz = get_zoom_point(i);

        // z = x+y*i , z0 = x0+y0*i , dz = dz_x+dz_y*i
        // x+y*i dz_x+dz_y*i  <<<      <<<    x0+y0*i
        //   |        |        |        |        |
        // (a+bi) * (c+di) + (a+bi) * (a+bi) + (e+fi)
        // z = z * dz + z * z + z0
        // = ac + adi + cbi - bd + aa + abi + abi - bb + e + fi
        // = ac - bd + aa - bb + e + (ad + cb + ab + ab + f)i
        // = ac - bd + aa - bb + e + (ad + cb + 2ab + f)i
        // = (c + a)a - (d + b)b + e + (ad + cb + 2ab + f)i
        var tmp = (dz.x + z.x) * z.x - (dz.y + z.y) * z.y + z0.x;
        z.y = 2.0 * z.x * z.y + z.x * dz.y + z.y * dz.x + z0.y;
        z.x = tmp;

        var d = dz * 0.5 + z;
        if (d.x * d.x + d.y * d.y >= 512.0) {
            break;
        }
    }

    return i;
}

@vertex
fn vs_main(vin: VertexInput) -> FragmentInput {
	var fin: FragmentInput;
	fin.pos = vec4<f32>(vin.pos, 0.0, 1.0);
	fin.z = 1.5 * vin.pos * vec2(ubo.aspect, 1.0) * ubo.zoom;
	return fin;
}

@fragment
fn fs_main(fin: FragmentInput) -> @location(0) vec4<f32> {
    // var i = mandelbrot(fin.z, 128u);
    var i = mandelbrot_2(fin.z);

    if (i == ubo.points) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }

    // 2 * pi / 3
    let mult: f32 = 2.09439510239319549231;

    // var v = sin(f32(128u - i)) * 0.5 + 0.5;
    // var v = fract(f32(i) * 0.1);
    var vf = f32(i) * 0.01;
    var v = vec3(sin(vf), sin(vf + mult), sin(vf + 2.0 * mult));


    return vec4(v, 1.0);
}
