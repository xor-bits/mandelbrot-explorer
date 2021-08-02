#version 420

layout (location = 0) out vec2 out_uv;

vec2 pos_a[6] = vec2[](
    vec2(-1.0, -1.0),
    vec2( 1.0,  1.0),
    vec2(-1.0,  1.0),

    vec2(-1.0, -1.0),
    vec2( 1.0, -1.0),
    vec2( 1.0,  1.0)
);

void main() {
	int i = gl_VertexIndex;
	gl_Position = vec4(pos_a[i], 0.0, 1.0);
	out_uv = pos_a[i];
}