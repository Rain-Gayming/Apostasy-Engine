#version 450
layout(location = 0) in uint vertexData;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
    vec4 offset;  
} pc;

void main() {
}
