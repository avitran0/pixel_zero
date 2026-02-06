precision mediump float;

varying vec2 v_texcoord;

uniform vec2 u_screen_size;
uniform sampler2D u_texture;

float width = 320.0;
float height = 240.0;
vec4 letterbox_color = vec4(0.0, 0.0, 0.0, 1.0);

void main() {
    float target_aspect = width / height;
    float screen_aspect = u_screen_size.x / u_screen_size.y;

    vec2 uv = v_texcoord;

    if (screen_aspect > target_aspect) {
        // screen is wider than target
        float scale = screen_aspect / target_aspect;
        uv.x = (uv.x - 0.5) / scale + 0.5;

        if (uv.x < 0.0 || uv.x > 1.0) {
            gl_FragColor = letterbox_color;
            return;
        }
    } else {
        // screen is taller than target
        float scale = target_aspect / screen_aspect;
        uv.y = (uv.y - 0.5) / scale + 0.5;

        if (uv.y < 0.0 || uv.y > 1.0) {
            gl_FragColor = letterbox_color;
            return;
        }
    }

    gl_FragColor = texture2D(u_texture, uv);
}
