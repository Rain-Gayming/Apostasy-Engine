#version 450

layout(location = 0) out vec4 outColor;

void main() {
    vec3 color = vec3(0.8, 0.8, 0.8);
    outColor = vec4(color, 1.0);
}
