
#version 450

layout(location = 0) in ivec3 inPosition;

layout(location = 0) out vec3 fragColor;

mat4 model = mat4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

layout(push_constant) uniform Push {
    mat4 view;
    mat4 proj;
} pc;

void main() {
    vec3 position = vec3(inPosition);

    gl_Position = pc.proj * pc.view * model * vec4(position, 1.0);
    fragColor = vec3(0.1, 0.1, 0.1);
}
