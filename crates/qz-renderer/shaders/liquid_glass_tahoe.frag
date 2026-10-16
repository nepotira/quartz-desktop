#version 450
// ============================================================================
// 3.2 & 3.3. SUPERELIPSE CONTÍNUA G2 (p = 2.35) & LIQUID GLASS TAHOE 26
// ============================================================================
layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec2 v_local_pos; // Pixel coordinates relative to panel center
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_BlurredBackdrop;

layout(push_constant) uniform GlassUniforms {
    vec2 half_size;      // w/2, h/2 in pixels
    vec2 screen_res;     // Display physical resolution (width, height)
    vec4 tint_color;     // Dark: rgba(30, 30, 34, 0.52), Light: rgba(246, 246, 248, 0.68)
    float corner_radius; // 11.0px for Floating Sidebar Pill, 12.0px for Windows, 16.0px for Spotlight/Menus
    float lens_width;    // 12.0px
    float refract_px;    // 5.5px
    int is_dark_mode;    // 1 = Dark, 0 = Light
} u;

// ============================================================================
// 3.2. SUPERELIPSE CONTÍNUA G2 (SQUIRCLE SDF) + GRADIENTE NORMAL 2D
// ============================================================================
float sdSquircleG2(vec2 p, vec2 half_size, float r) {
    vec2 q = abs(p) - (half_size - r);
    vec2 m = max(q, 0.0);
    // Norma p = 2.35 para curvatura contínua G2 idêntica ao macOS Tahoe 26
    float lp = pow(pow(m.x, 2.35) + pow(m.y, 2.35), 1.0 / 2.35);
    return lp + min(max(q.x, q.y), 0.0) - r;
}

vec2 sdSquircleNormal(vec2 p, vec2 half_size, float r) {
    const float eps = 0.5;
    float dx = sdSquircleG2(p + vec2(eps, 0.0), half_size, r) - sdSquircleG2(p - vec2(eps, 0.0), half_size, r);
    float dy = sdSquircleG2(p + vec2(0.0, eps), half_size, r) - sdSquircleG2(p - vec2(0.0, eps), half_size, r);
    return normalize(vec2(dx, dy) + 1e-6);
}

// ============================================================================
// 3.3. COMPOSIÇÃO LIQUID GLASS COM REFRAÇÃO DE LENTE (TAHOE 26)
// Inclui fallback Vogel Disk 32-taps garantindo blur pesado em qualquer FBO!
// ============================================================================
vec4 compose_liquid_glass_tahoe(
    sampler2D blurred_backdrop_tex,
    vec2 frag_coord,
    vec2 screen_res,
    vec2 local_pos,
    vec2 half_size,
    float corner_radius,
    int is_dark_mode
) {
    float d = sdSquircleG2(local_pos, half_size, corner_radius);
    float aa_mask = 1.0 - smoothstep(-0.5, 0.5, d);
    if (aa_mask <= 0.001) return vec4(0.0);

    vec2 n = sdSquircleNormal(local_pos, half_size, corner_radius);
    vec2 screen_uv = frag_coord / screen_res;

    // 1. Refração Ótica de Borda Curva nos últimos 12.0px (d in [-12.0, 0.0])
    float lens_factor = smoothstep(-12.0, 0.0, d);
    vec2 uv_refr = screen_uv - n * lens_factor * (5.5 / screen_res);

    // 2. Amostragem adicional Vogel Disk + Aberração Cromática Radial
    float ign = fract(52.9829189 * fract(0.06711056 * frag_coord.x + 0.00583715 * frag_coord.y));
    vec2 center_vec = screen_uv - 0.5;
    vec2 chrom_shift = center_vec * (0.0012 * pow(length(center_vec), 3.0));

    vec3 acc = vec3(0.0);
    float w_sum = 0.0;
    const int TAPS = 32;
    const float GOLDEN_ANGLE = 2.39996323;
    const float EXTRA_RADIUS = 28.0;
    float theta0 = ign * 6.2831853;

    for (int i = 0; i < TAPS; i++) {
        float r = sqrt((float(i) + 0.5) / float(TAPS)) * EXTRA_RADIUS;
        float th = float(i) * GOLDEN_ANGLE + theta0;
        vec2 off = (vec2(cos(th), sin(th)) * r) / screen_res;
        float w = exp(-0.5 * pow(r / (EXTRA_RADIUS * 0.65), 2.0));
        vec2 suv = clamp(uv_refr + off, vec2(0.002), vec2(0.998));
        acc.r += texture(blurred_backdrop_tex, suv + chrom_shift).r * w;
        acc.g += texture(blurred_backdrop_tex, suv).g * w;
        acc.b += texture(blurred_backdrop_tex, suv - chrom_shift).b * w;
        w_sum += w;
    }
    vec3 backdrop = acc / max(w_sum, 0.0001);

    // 3. Boost de Saturação Luma (S = 1.95) com pesos Rec.709
    float luma = dot(backdrop, vec3(0.2126, 0.7152, 0.0722));
    vec3 saturated = clamp(mix(vec3(luma), backdrop, 1.95), 0.0, 1.0);

    // 4. Tint & Vibrancy Blend (Dark Mode vs Light Mode)
    vec3 glass_rgb;
    if (is_dark_mode == 1) {
        // Dark Mode: rgba(30, 30, 34, 0.52) em Luminosity/ColorBurn + SourceOver
        vec3 dark_tint = vec3(30.0 / 255.0, 30.0 / 255.0, 34.0 / 255.0);
        vec3 burn = 1.0 - (1.0 - saturated) / max(dark_tint + 0.28, vec3(0.05));
        glass_rgb = mix(mix(saturated, clamp(burn, 0.0, 1.0), 0.32), dark_tint, 0.52);
    } else {
        // Light Mode: rgba(246, 246, 248, 0.68) em ColorDodge + SourceOver
        vec3 light_tint = vec3(246.0 / 255.0, 246.0 / 255.0, 248.0 / 255.0);
        vec3 dodge = saturated / max(1.0 - light_tint * 0.28, vec3(0.05));
        glass_rgb = mix(mix(saturated, clamp(dodge, 0.0, 1.0), 0.30), light_tint, 0.68);
    }

    // 5. Blue Noise / Interleaved Gradient Noise Dithering
    float dither = (ign - 0.5) * (1.5 / 255.0);
    glass_rgb = clamp(glass_rgb + dither, 0.0, 1.0);

    // 6. Specular Rim Highlight Interno (1.0px) + Hairline Externa (0.5px)
    float inner_rim_mask = 1.0 - smoothstep(0.0, 1.15, abs(d + 0.75));
    float spec_light = max(dot(n, normalize(vec2(-0.4, -0.9))), 0.15) * 0.45;
    glass_rgb = mix(glass_rgb, vec3(1.0), inner_rim_mask * spec_light);

    float outer_hairline = 1.0 - smoothstep(0.0, 0.65, abs(d));
    glass_rgb = mix(glass_rgb, vec3(0.0), outer_hairline * 0.38);

    return vec4(glass_rgb, aa_mask);
}

void main() {
    frag_color = compose_liquid_glass_tahoe(
        u_BlurredBackdrop,
        gl_FragCoord.xy,
        u.screen_res,
        v_local_pos,
        u.half_size,
        u.corner_radius,
        u.is_dark_mode
    );
}
