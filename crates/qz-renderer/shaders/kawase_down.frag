#version 450
// ============================================================================
// 3.1. PIRÂMIDE DUAL-KAWASE BLUR — DOWNSAMPLE PASS (5 LEVELS: 1/2 .. 1/32)
// ============================================================================
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_tex;

layout(push_constant) uniform PushConstants {
    vec2 half_pixel; // 0.5 / vec2(textureSize(u_tex, 0))
    float offset;    // Expansion offset
} pc;

vec4 kawase_downsample(sampler2D tex, vec2 uv, vec2 half_pixel, float offset) {
    vec2 d = half_pixel * offset;
    vec4 sum = texture(tex, uv) * 4.0;
    sum += texture(tex, uv - d);
    sum += texture(tex, uv + d);
    sum += texture(tex, uv + vec2(d.x, -d.y));
    sum += texture(tex, uv - vec2(d.x, -d.y));
    return sum * 0.125;
}

void main() {
    frag_color = kawase_downsample(u_tex, v_uv, pc.half_pixel, pc.offset);
}
