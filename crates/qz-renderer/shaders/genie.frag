#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_WindowTexture;

layout(push_constant) uniform GenieFragUniforms {
    layout(offset = 48) float opacity; // Fade out slightly at end of genie animation
} pc;

void main() {
    vec4 color = texture(u_WindowTexture, v_uv);
    frag_color = color * pc.opacity;
}
