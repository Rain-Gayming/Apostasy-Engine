#version 450
layout(location = 0) in uint vertexData;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
    vec3 offset;  
} pc;

void main() {
    int posZ = int((vertexData >> 12) & 63u);
    int posY = int((vertexData >> 6) & 63u);
    int posX = int(vertexData & 63u);
    vec3 pos = vec3(float(posX), float(posY), float(posZ)) + pc.offset;  
    gl_Position = pc.mvp * vec4(pos, 1.0);
}
