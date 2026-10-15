#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_tex;

layout(push_constant) uniform PushConstants {
    vec2 half_pixel;
    float offset;
} pc;

void main() {
    vec2 d = pc.half_pixel * pc.offset;
    vec4 sum = vec4(0.0);
    sum += texture(u_tex, v_uv + vec2(-d.x * 2.0, 0.0));
    sum += texture(u_tex, v_uv + vec2( d.x * 2.0, 0.0));
    sum += texture(u_tex, v_uv + vec2(0.0, -d.y * 2.0));
    sum += texture(u_tex, v_uv + vec2(0.0,  d.y * 2.0));
    sum += texture(u_tex, v_uv + vec2(-d.x,  d.y)) * 2.0;
    sum += texture(u_tex, v_uv + vec2( d.x,  d.y)) * 2.0;
    sum += texture(u_tex, v_uv + vec2(-d.x, -d.y)) * 2.0;
    sum += texture(u_tex, v_uv + vec2( d.x, -d.y)) * 2.0;
    frag_color = sum / 12.0;
}
