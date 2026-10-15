#version 450
layout(location = 0) in vec3 a_position; // 3D cube face vertex (-1 to +1)
layout(location = 1) in vec2 a_uv;
layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec3 v_normal;

layout(push_constant) uniform CubeUniforms {
    mat4 mvp_matrix;
    float rotation_angle;
    int active_face;
} u;

void main() {
    v_uv = a_uv;
    v_normal = a_position;
    gl_Position = u.mvp_matrix * vec4(a_position, 1.0);
}
