#version 450
// ============================================================================
// 3.4. SOMBRA ANALÍTICA DE CAMADA DUPLA (ABRAMOWITZ & STEGUN ERF)
// Renderizada em um quad expandido +90px fora da janela (Lei 3 Anti-Fraude)
// ============================================================================
layout(location = 0) in vec2 v_local_pos;
layout(location = 0) out vec4 frag_color;

layout(push_constant) uniform ShadowUniforms {
    vec2 half_size;      // Window half-dimensions in pixels (w/2, h/2)
    float corner_radius; // G2 squircle corner radius
    int is_active;       // 1 = Focused Window, 0 = Unfocused Window, 2 = Floating Glass Pill
} u;

vec2 erf_approx(vec2 x) {
    vec2 s = sign(x), a = abs(x);
    x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    x *= x;
    return s - s / (x * x);
}

float shadow_box_integral_x(float x, float y, float sigma, float r, vec2 b) {
    float delta = min(b.y - r - abs(y), 0.0);
    float curved = b.x - r + sqrt(max(0.0, r * r - delta * delta));
    vec2 integral = 0.5 + 0.5 * erf_approx((x + vec2(-curved, curved)) * (0.70710678 / sigma));
    return integral.y - integral.x;
}

float gaussian_box_shadow(vec2 p, vec2 b, float r, float sigma, float dy) {
    vec2 q = p - vec2(0.0, dy);
    float low = q.y - b.y, high = q.y + b.y;
    float start = clamp(-3.0 * sigma, low, high);
    float end   = clamp( 3.0 * sigma, low, high);
    float step_y = (end - start) * 0.25;
    float y = start + step_y * 0.5;
    float sum = 0.0;
    for (int i = 0; i < 4; i++) {
        sum += shadow_box_integral_x(q.x, q.y - y, sigma, r, b) * exp(-(y * y) / (2.0 * sigma * sigma));
        y += step_y;
    }
    return sum * step_y / (2.50662827 * sigma);
}

vec4 eval_window_dual_shadow(vec2 local_pos, vec2 half_size, float r, int is_focused) {
    if (is_focused == 2) {
        // Floating Glass Pill Sidebar independent shadow
        float s_pill = gaussian_box_shadow(local_pos, half_size, r, 10.0, 4.0) * 0.22;
        return vec4(0.0, 0.0, 0.0, clamp(s_pill, 0.0, 1.0));
    }
    float ao_a    = (is_focused == 1) ? 0.30 : 0.18;
    float umbra_a = (is_focused == 1) ? 0.52 : 0.20;
    float s1 = gaussian_box_shadow(local_pos, half_size, r, 4.0,  2.0) * ao_a;
    float s2 = gaussian_box_shadow(local_pos, half_size, r, 42.0, 24.0) * umbra_a;
    return vec4(0.0, 0.0, 0.0, clamp(s1 + s2, 0.0, 1.0));
}

void main() {
    frag_color = eval_window_dual_shadow(v_local_pos, u.half_size, u.corner_radius, u.is_active);
}
