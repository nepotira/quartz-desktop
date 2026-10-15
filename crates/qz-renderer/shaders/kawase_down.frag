#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_tex;

layout(push_constant) uniform PushConstants {
    vec2 half_pixel; // 0.5 / vec2(textureSize(u_tex, 0))
    float offset;    // 1.5
} pc;

void main() {
    vec2 d = pc.half_pixel * pc.offset;
    vec4 sum = texture(u_tex, v_uv) * 4.0;
    sum += texture(u_tex, v_uv - d);
    sum += texture(u_tex, v_uv + d);
    sum += texture(u_tex, v_uv + vec2(d.x, -d.y));
    sum += texture(u_tex, v_uv - vec2(d.x, -d.y));
    frag_color = sum * 0.125;
}
