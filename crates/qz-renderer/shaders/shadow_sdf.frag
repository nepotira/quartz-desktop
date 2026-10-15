#version 450
layout(location = 0) in vec2 v_local_pos;
layout(location = 0) out vec4 frag_color;

layout(push_constant) uniform ShadowUniforms {
    vec2 half_size;      // Window half-dimensions in pixels (w/2, h/2)
    float corner_radius; // G2 squircle corner radius
    int is_active;       // 1 = Active Window, 0 = Inactive Window, 2 = Floating Glass Pill
} u;

// High-precision erf(x) approximation for analytical Gaussian integration of rounded box
vec2 erf_approx(vec2 x) {
    vec2 s = sign(x), a = abs(x);
    x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    x *= x;
    return s - s / (x * x);
}

float rounded_box_shadow_x(float x, float y, float sigma, float corner, vec2 half_size) {
    float delta = min(half_size.y - corner - abs(y), 0.0);
    float curved = half_size.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
    vec2 integral = 0.5 + 0.5 * erf_approx((x + vec2(-curved, curved)) * (sqrt(0.5) / sigma));
    return integral.y - integral.x;
}

float eval_gaussian_shadow(vec2 pos, vec2 half_size, float corner, float sigma, float y_offset) {
    vec2 p = pos - vec2(0.0, y_offset);
    float low = p.y - half_size.y;
    float high = p.y + half_size.y;
    float start = clamp(-3.0 * sigma, low, high);
    float end = clamp(3.0 * sigma, low, high);

    float step = (end - start) / 4.0;
    float y = start + step * 0.5;
    float val = 0.0;
    for (int i = 0; i < 4; i++) {
        val += rounded_box_shadow_x(p.x, p.y - y, sigma, corner, half_size) * exp(-y * y / (2.0 * sigma * sigma));
        y += step;
    }
    return val * step / (sqrt(2.0 * 3.14159265) * sigma);
}

void main() {
    float alpha = 0.0;
    if (u.is_active == 2) {
        // Floating Glass Pill shadow (Tahoe 26 Sidebar / Toolbar)
        alpha = eval_gaussian_shadow(v_local_pos, u.half_size, u.corner_radius, 10.0, 4.0) * 0.22;
    } else if (u.is_active == 1) {
        // Active Window: Ambient Occlusion + Deep Atmospheric Penumbra
        float ao = eval_gaussian_shadow(v_local_pos, u.half_size, u.corner_radius, 4.0, 2.0) * 0.28;
        float umbra = eval_gaussian_shadow(v_local_pos, u.half_size, u.corner_radius, 38.0, 22.0) * 0.42;
        alpha = clamp(ao + umbra, 0.0, 1.0);
    } else {
        // Inactive Window: Restrained AO and shallower penumbra
        float ao = eval_gaussian_shadow(v_local_pos, u.half_size, u.corner_radius, 4.0, 2.0) * 0.18;
        float umbra = eval_gaussian_shadow(v_local_pos, u.half_size, u.corner_radius, 30.0, 14.0) * 0.16;
        alpha = clamp(ao + umbra, 0.0, 1.0);
    }
    frag_color = vec4(0.0, 0.0, 0.0, alpha);
}
