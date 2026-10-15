#version 450
layout(location = 0) in vec2 a_uv; // Grid coordinates (u, v) in [0, 1]^2 on 64x64 mesh
layout(location = 0) out vec2 v_uv;

layout(push_constant) uniform GenieUniforms {
    vec4 win_rect;   // Window rectangle: (x, y, width, height)
    vec4 icon_rect;  // Dock icon rectangle: (x, y, width, height)
    vec2 screen_res; // Display resolution: (width, height)
    float t;         // Normalized spring progress [0.0, 1.0]
    int dock_edge;   // 0 = Bottom, 1 = Left, 2 = Right
} u;

float hermite(float x) {
    x = clamp(x, 0.0, 1.0);
    return x * x * (3.0 - 2.0 * x);
}

void main() {
    v_uv = a_uv;
    float p1 = smoothstep(0.0, 0.65, u.t);
    float p2 = smoothstep(0.25, 1.0, u.t);

    vec2 uv = (u.dock_edge == 0) ? a_uv : a_uv.yx;
    vec4 w_rect = (u.dock_edge == 0) ? u.win_rect : u.win_rect.yxwz;
    vec4 i_rect = (u.dock_edge == 0) ? u.icon_rect : u.icon_rect.yxwz;

    float v = (u.dock_edge == 1) ? (1.0 - uv.y) : uv.y;
    float local_w = clamp((v - (1.0 - p1)) / max(p1, 0.0001), 0.0, 1.0);
    float blend_x = mix(hermite(local_w), 1.0, p2);

    float xl = mix(w_rect.x, i_rect.x, blend_x);
    float xr = mix(w_rect.x + w_rect.z, i_rect.x + i_rect.z, blend_x);

    float x = mix(xl, xr, uv.x);
    float y_win = w_rect.y + uv.y * w_rect.w;
    float y_icn = i_rect.y + uv.y * i_rect.w;
    float y = mix(y_win, y_icn, p2 * (0.3 + 0.7 * hermite(v)));

    vec2 final_pos = (u.dock_edge == 0) ? vec2(x, y) : vec2(y, x);
    vec2 ndc = (final_pos / u.screen_res) * 2.0 - 1.0;
    gl_Position = vec4(ndc, 0.0, 1.0);
}
