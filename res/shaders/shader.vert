
#version 450

layout(location = 0) in uvec3 inPosition; // Change to uvec3 for u8 data

layout(location = 1) out vec3 fragColor;

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
    // Convert u8 to normalized coordinates (0-255 -> 0.0-1.0) or scale as needed
    vec3 position = vec3(inPosition) / 255.0;

    gl_Position = pc.proj * pc.view * model * vec4(position, 1.0);
    fragColor = vec3(1.0, 1.0, 1.0);
}
