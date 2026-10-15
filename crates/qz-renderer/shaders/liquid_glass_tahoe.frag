#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec2 v_local_pos; // Pixel coordinates relative to panel center
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_BlurredBackdrop;

layout(push_constant) uniform GlassUniforms {
    vec2 half_size;      // w/2, h/2 in pixels
    vec2 screen_res;     // Display physical resolution (width, height)
    vec4 tint_color;     // Dark: (0.11, 0.11, 0.13, 0.66), Light: (0.95, 0.95, 0.97, 0.70)
    float corner_radius; // 12.0px for windows/pills, 16.0px for Spotlight/Menus
    float lens_width;    // 14.0px
    float refract_px;    // 6.5px
    int is_dark_mode;    // 1 = Dark, 0 = Light
} u;

// SDF Superellipse G2 (Squircle L4 norm)
float sd_squircle(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - (b - r);
    vec2 m = max(q, 0.0);
    float l4 = pow(pow(m.x, 4.0) + pow(m.y, 4.0), 0.25);
    return l4 + min(max(q.x, q.y), 0.0) - r;
}

// Analytical Gradient / 2D Normal of Squircle
vec2 sd_squircle_normal(vec2 p, vec2 b, float r) {
    const float eps = 0.5;
    float dx = sd_squircle(p + vec2(eps, 0.0), b, r) - sd_squircle(p - vec2(eps, 0.0), b, r);
    float dy = sd_squircle(p + vec2(0.0, eps), b, r) - sd_squircle(p - vec2(0.0, eps), b, r);
    return normalize(vec2(dx, dy) + 1e-6);
}

// Interleaved Gradient Noise (IGN) for dithering and anti-banding
float ign_noise(vec2 frag_coord) {
    vec3 magic = vec3(0.06711056, 0.00583715, 52.9829189);
    return fract(magic.z * fract(dot(frag_coord, magic.xy)));
}

void main() {
    float d = sd_squircle(v_local_pos, u.half_size, u.corner_radius);
    float aa_mask = 1.0 - smoothstep(-0.5, 0.5, d);
    if (aa_mask <= 0.001) discard;

    vec2 normal = sd_squircle_normal(v_local_pos, u.half_size, u.corner_radius);

    // 1. Edge Refractive Lens Profile (macOS Tahoe 26)
    float edge_factor = clamp(-d / max(u.lens_width, 0.001), 0.0, 1.0);
    float eta = pow(1.0 - edge_factor, 3.0);
    vec2 refract_offset = -normal * eta * (u.refract_px / u.screen_res);

    // 2. Chromatic Dispersion (Edge + Radial Aberration)
    vec2 center_dist = v_uv - 0.5;
    float radial_aberration = 0.0012 * pow(length(center_dist), 3.0);
    vec2 uv_r = v_uv + refract_offset * 1.018 + center_dist * radial_aberration;
    vec2 uv_g = v_uv + refract_offset * 1.000;
    vec2 uv_b = v_uv + refract_offset * 0.982 - center_dist * radial_aberration;

    vec3 backdrop = vec3(
        texture(u_BlurredBackdrop, uv_r).r,
        texture(u_BlurredBackdrop, uv_g).g,
        texture(u_BlurredBackdrop, uv_b).b
    );

    // 3. Saturation Boost in Rec.709 Luma Space (S = 1.85)
    const vec3 luma_weights = vec3(0.2126, 0.7152, 0.0722);
    float luma = dot(backdrop, luma_weights);
    vec3 saturated = mix(vec3(luma), backdrop, 1.85);
    saturated = clamp(saturated, 0.0, 1.0);

    // 4. Vibrancy Blend (Color Burn in Dark Mode / Smooth Color Dodge in Light Mode)
    vec3 blended;
    if (u.is_dark_mode == 1) {
        vec3 burn = 1.0 - (1.0 - saturated) / max(u.tint_color.rgb + 0.25, 0.05);
        blended = mix(saturated, clamp(burn, 0.0, 1.0), 0.35);
        blended = mix(blended, u.tint_color.rgb, u.tint_color.a);
    } else {
        vec3 dodge = saturated / max(1.0 - u.tint_color.rgb * 0.3, 0.05);
        blended = mix(saturated, clamp(dodge, 0.0, 1.0), 0.30);
        blended = mix(blended, u.tint_color.rgb, u.tint_color.a);
    }

    // 5. Specular Rim Highlight of 1px reacting to light direction and background luma
    vec2 light_dir = normalize(vec2(-0.35, -0.936));
    float rim_mask = 1.0 - smoothstep(0.0, 1.25, abs(d + 0.5));
    float spec_intensity = max(0.15, dot(normal, light_dir)) + 0.35 * luma;
    vec3 rim_color = mix(blended, vec3(1.0), rim_mask * spec_intensity * 0.55);

    // 6. Procedural Anti-Banding Dithering
    float dither = (ign_noise(gl_FragCoord.xy) - 0.5) * (3.0 / 255.0);
    vec3 final_rgb = clamp(rim_color + dither, 0.0, 1.0);

    frag_color = vec4(final_rgb, aa_mask);
}
