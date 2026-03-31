#version 450

layout(set = 0, binding = 1) uniform sampler2D skySampler;

layout(location = 0) in vec3 worldDirection;
layout(location = 0) out vec4 outColor;

const float PI = 3.14159265358979323846;

void main() {
    vec3 d = normalize(worldDirection);
    float u = atan(d.z, d.x) / (2.0 * PI) + 0.5;
    float v = 0.5 - asin(d.y) / PI;
    outColor = texture(skySampler, vec2(u, v));
}
