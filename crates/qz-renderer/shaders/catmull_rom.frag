#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_SourceTexture;

layout(push_constant) uniform FilterUniforms {
    vec2 source_res; // Source texture dimensions (e.g. @2x offscreen framebuffer)
    vec2 target_res; // Physical display viewport dimensions
} u;

// Catmull-Rom 9-tap bicubic downsampling filter
vec4 sample_catmull_rom_9tap(sampler2D tex, vec2 uv, vec2 tex_size) {
    vec2 sample_pos = uv * tex_size;
    vec2 tc = floor(sample_pos - 0.5) + 0.5;
    vec2 f = sample_pos - tc;
    vec2 f2 = f * f;
    vec2 f3 = f2 * f;

    // Catmull-Rom weights (B = 0, C = 0.5)
    vec2 w0 = f2 - 0.5 * (f3 + f);
    vec2 w1 = 1.5 * f3 - 2.5 * f2 + 1.0;
    vec2 w3 = 0.5 * (f3 - f2);
    vec2 w2 = 1.0 - w0 - w1 - w3;

    vec2 w12 = w1 + w2;
    vec2 tc12 = tc + (w2 / w12);
    vec2 tc0 = tc - 1.0;
    vec2 tc3 = tc + 2.0;

    vec2 inv_tex_size = 1.0 / tex_size;

    vec4 color = vec4(0.0);
    // 9-sample bilinear reconstruction
    color += texture(tex, vec2(tc0.x,  tc0.y)  * inv_tex_size) * (w0.x * w0.y);
    color += texture(tex, vec2(tc12.x, tc0.y)  * inv_tex_size) * (w12.x * w0.y);
    color += texture(tex, vec2(tc3.x,  tc0.y)  * inv_tex_size) * (w3.x * w0.y);

    color += texture(tex, vec2(tc0.x,  tc12.y) * inv_tex_size) * (w0.x * w12.y);
    color += texture(tex, vec2(tc12.x, tc12.y) * inv_tex_size) * (w12.x * w12.y);
    color += texture(tex, vec2(tc3.x,  tc12.y) * inv_tex_size) * (w3.x * w12.y);

    color += texture(tex, vec2(tc0.x,  tc3.y)  * inv_tex_size) * (w0.x * w3.y);
    color += texture(tex, vec2(tc12.x, tc3.y)  * inv_tex_size) * (w12.x * w3.y);
    color += texture(tex, vec2(tc3.x,  tc3.y)  * inv_tex_size) * (w3.x * w3.y);

    return max(color, vec4(0.0));
}

void main() {
    frag_color = sample_catmull_rom_9tap(u_SourceTexture, v_uv, u.source_res);
}
