#version 450
// ============================================================================
// 3.5. EFEITO GÊNIO (GENIE MESH WARPING 64x64 CÉLULAS / 65x65 VÉRTICES)
// ============================================================================
layout(location = 0) in vec2 a_uv; // Grid coordinates (u, v) in [0, 1]^2 on 64x64 mesh
layout(location = 0) out vec2 v_uv;

layout(push_constant) uniform GenieUniforms {
    vec4 win_rect;   // Window rectangle: (x, y, width, height)
    vec4 icon_rect;  // Dock icon rectangle: (x, y, width, height)
    vec2 screen_res; // Display resolution: (width, height)
    float t;         // Normalized spring progress [0.0, 1.0]
    int dock_edge;   // 0 = Bottom, 1 = Left, 2 = Right
} u;

vec2 eval_genie_vertex(vec2 uv, vec4 win_rect, vec4 icon_rect, float t) {
    float p1 = smoothstep(0.0, 0.65, t);
    float p2 = smoothstep(0.25, 1.0, t);
    float v = uv.y;
    float w = clamp((v - (1.0 - p1)) / (p1 + 1e-4), 0.0, 1.0);
    float h_w = w * w * (3.0 - 2.0 * w);
    float h_v = v * v * (3.0 - 2.0 * v);
    float blend_x = mix(h_w, 1.0, p2);
    float xl = mix(win_rect.x, icon_rect.x, blend_x);
    float xr = mix(win_rect.x + win_rect.z, icon_rect.x + icon_rect.z, blend_x);
    float x = mix(xl, xr, uv.x);
    float y = mix(win_rect.y + v * win_rect.w, icon_rect.y + v * icon_rect.w, p2 * (0.3 + 0.7 * h_v));
    return vec2(x, y);
}

void main() {
    v_uv = a_uv;
    vec2 uv_oriented = (u.dock_edge == 0) ? a_uv : a_uv.yx;
    vec4 w_rect = (u.dock_edge == 0) ? u.win_rect : u.win_rect.yxwz;
    vec4 i_rect = (u.dock_edge == 0) ? u.icon_rect : u.icon_rect.yxwz;

    vec2 warped = eval_genie_vertex(uv_oriented, w_rect, i_rect, u.t);
    vec2 final_pos = (u.dock_edge == 0) ? warped : warped.yx;
    vec2 ndc = (final_pos / u.screen_res) * 2.0 - 1.0;
    gl_Position = vec4(ndc, 0.0, 1.0);
}
