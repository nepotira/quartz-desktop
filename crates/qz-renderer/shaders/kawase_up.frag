#version 450
// ============================================================================
// 3.1. PIRÂMIDE DUAL-KAWASE BLUR — UPSAMPLE PASS (5 LEVELS: 1/32 .. 1/1, sigma >= 48px)
// ============================================================================
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_tex;

layout(push_constant) uniform PushConstants {
    vec2 half_pixel;
    float offset;
} pc;

vec4 kawase_upsample(sampler2D tex, vec2 uv, vec2 half_pixel, float offset) {
    vec2 d = half_pixel * offset;
    vec4 sum = vec4(0.0);
    sum += texture(tex, uv + vec2(-2.0 * d.x, 0.0));
    sum += texture(tex, uv + vec2( 2.0 * d.x, 0.0));
    sum += texture(tex, uv + vec2(0.0, -2.0 * d.y));
    sum += texture(tex, uv + vec2(0.0,  2.0 * d.y));
    sum += texture(tex, uv + vec2(-d.x,  d.y)) * 2.0;
    sum += texture(tex, uv + vec2( d.x,  d.y)) * 2.0;
    sum += texture(tex, uv + vec2(-d.x, -d.y)) * 2.0;
    sum += texture(tex, uv + vec2( d.x, -d.y)) * 2.0;
    return sum / 12.0;
}

void main() {
    frag_color = kawase_upsample(u_tex, v_uv, pc.half_pixel, pc.offset);
}
