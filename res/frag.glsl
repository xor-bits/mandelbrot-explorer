#version 420

layout (location = 0) in vec2 in_uv;
layout (location = 0) out vec4 out_col;

layout (binding = 0) uniform UBO {
	dvec2 draw_top_left;
	dvec2 draw_bottom_right;

	dvec2 select_top_left;
	dvec2 select_bottom_right;

	uint iterations;
} ubo;

int mandelbrot(dvec2 c, uint maximum) {
	int i = 0;
	dvec2 z = c;
	double zx = c.x;
	double zy = c.y;

	//Mandelbrot
    while (z.x * z.x + z.y * z.y <= 4.0 && i <= maximum) {
        double ytemp = 2 * z.x * z.y + c.y; 
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
	dvec2 uv = mix(ubo.draw_top_left, ubo.draw_bottom_right, in_uv * 0.5 + 0.5);
	
	// iterations
	float v = float(mandelbrot(uv, ubo.iterations));
	
	// coloring
	v = -cos(v / 10.0) * 0.5 + 0.5;
	// v /= float(ubo.iterations);

	// selection
	double left = min(ubo.select_top_left.x, ubo.select_bottom_right.x);
	double right = max(ubo.select_top_left.x, ubo.select_bottom_right.x);
	double top = min(ubo.select_top_left.y, ubo.select_bottom_right.y);
	double bottom = max(ubo.select_top_left.y, ubo.select_bottom_right.y);

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