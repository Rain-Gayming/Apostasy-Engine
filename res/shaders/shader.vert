#version 450

layout(location = 0) out vec3 fragColor;

vec2 positions[6] = vec2[](
        vec2(0.0, 0.0),
        vec2(0.0, 1.0),
        vec2(1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, 0.0)
    );

vec3 colors[6] = vec3[](
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 1.0, 0.0),
        vec3(1.0, 0.0, 0.0)
    );

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

layout(location = 0) in vec3 inPosition;

void main() {
    // apply model, view, then projection
    gl_Position = pc.proj * pc.view * model * vec4(positions[gl_VertexIndex], 0.0, 1.0);
    fragColor = colors[gl_VertexIndex];
}

