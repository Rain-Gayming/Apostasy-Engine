
#version 450

layout(location = 0) in int data;

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
    int posZ = (data >> 12) & 63;
    int posY = (data >> 6) & 63;
    int posX = data & 63;

    // get the position, and add the chunk space coordinate * 32 (the chunk size)
    vec3 position = vec3(vec3(posX, posY, posZ) + vec3(pc.chunk_pos * 32));
    gl_Position = pc.proj * pc.view * model * vec4(position, 1.0);
    fragColor = vec3(0.1, 0.1, 0.1);
}
