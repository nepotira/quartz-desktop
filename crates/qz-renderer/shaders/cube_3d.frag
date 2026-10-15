#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec3 v_normal;
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_SessionTexture;

layout(push_constant) uniform CubeFragUniforms {
    layout(offset = 72) float ambient_light;
    float shading_strength;
} u;

void main() {
    vec4 session_color = texture(u_SessionTexture, v_uv);

    // Subtle 3D perspective shading based on surface normal angle
    vec3 light_dir = normalize(vec3(0.3, 0.4, 1.0));
    float ndotl = max(dot(normalize(v_normal), light_dir), 0.0);
    float light_factor = mix(u.ambient_light, 1.0, ndotl * u.shading_strength);

    // Vignette towards cube corners
    vec2 center_dist = abs(v_uv - 0.5) * 2.0;
    float vignette = 1.0 - 0.15 * dot(center_dist, center_dist);

    frag_color = vec4(session_color.rgb * light_factor * vignette, session_color.a);
}
