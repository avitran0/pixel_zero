attribute vec2 a_position;
attribute vec2 a_texcoord;

varying vec2 v_texcoord;

uniform mat4 u_projection;
uniform vec2 u_position;
uniform vec2 u_size;

void main() {
    vec2 position = a_position * u_size + u_position;
    gl_Position = u_projection * vec4(position, 0.0, 1.0);
    v_texcoord = a_texcoord;
}
