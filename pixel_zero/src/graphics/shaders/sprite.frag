precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_texture;

void main() {
    vec4 tex_color = texture2D(u_texture, v_texcoord);

    // alpha blending
    if (tex_color.a < 0.01) {
        discard;
    }

    gl_FragColor = tex_color;
}
