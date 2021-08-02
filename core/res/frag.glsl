#version 420

// DEF_VEC2_TYPE
// DEF_FLOAT_TYPE

layout (location = 0) in vec2 in_uv;
layout (location = 0) out vec4 out_col;

layout (binding = 0) uniform UBO {
	DEF_VEC2_TYPE draw_top_left;
	DEF_VEC2_TYPE draw_bottom_right;

	DEF_VEC2_TYPE select_top_left;
	DEF_VEC2_TYPE select_bottom_right;

	uint iterations;
} ubo;

int mandelbrot(DEF_VEC2_TYPE c, uint maximum) {
	int i = 0;
	DEF_VEC2_TYPE z = c;
	DEF_FLOAT_TYPE zx = c.x;
	DEF_FLOAT_TYPE zy = c.y;

	//Mandelbrot
    while (z.x * z.x + z.y * z.y <= 4.0 && i <= maximum) {
        DEF_FLOAT_TYPE ytemp = 2 * z.x * z.y + c.y; 
		z.x = z.x * z.x - z.y * z.y + c.x;
        z.y = ytemp;
        
		if (z.x == c.y && z.y == c.y)
			return 0;
        
		i += 1;
    }
    
	if (i >= maximum)
		return 0;
	else
		return i;
}

void main() {
	// position
	DEF_VEC2_TYPE uv = mix(ubo.draw_top_left, ubo.draw_bottom_right, in_uv * 0.5 + 0.5);
	
	// iterations
	float v = float(mandelbrot(uv, ubo.iterations));
	
	// coloring
	v = -cos(v / 10.0) * 0.5 + 0.5;
	// v /= float(ubo.iterations);

	// selection
	DEF_FLOAT_TYPE left = min(ubo.select_top_left.x, ubo.select_bottom_right.x);
	DEF_FLOAT_TYPE right = max(ubo.select_top_left.x, ubo.select_bottom_right.x);
	DEF_FLOAT_TYPE top = min(ubo.select_top_left.y, ubo.select_bottom_right.y);
	DEF_FLOAT_TYPE bottom = max(ubo.select_top_left.y, ubo.select_bottom_right.y);

	if (uv.x < left
	 || uv.x > right
	 || uv.y < top
	 || uv.y > bottom) 
	{
		v *= 0.3;
	}

	// color
	out_col = vec4(v, v, v, 0.0);
}