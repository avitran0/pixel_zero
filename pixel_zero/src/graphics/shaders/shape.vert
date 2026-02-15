attribute vec2 a_position;

uniform mat4 u_projection;
uniform vec2 u_position;
uniform vec2 u_size;
uniform vec2 u_line_unit;
uniform vec2 u_line_normal;
uniform float u_line_length;

void main() {
    vec2 local = a_position;
    vec2 position = u_position
        + u_line_unit * (local.x * u_line_length)
        + u_line_normal * local.y;
    gl_Position = u_projection * vec4(position, 0.0, 1.0);
}
