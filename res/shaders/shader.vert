
#version 450

layout(location = 0) in uvec3 inPosition;

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
    ivec3 chunk_pos;
} pc;

void main() {
    // get the position, and add the chunk space coordinate * 32 (the chunk size)
    vec3 position = vec3(inPosition + (pc.chunk_pos * 32));
    gl_Position = pc.proj * pc.view * model * vec4(position, 1.0);
    fragColor = vec3(0.1, 0.1, 0.1);
}
