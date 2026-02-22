#version 450
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;


layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
    vec4 offset;
} pc;

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec2 fragTexCoord;

void main() {
    vec3 offset = vec3(pc.offset.x, pc.offset.y, pc.offset.z);
    gl_Position = pc.mvp * vec4(inPosition + offset, 1.0);
    fragNormal = normalize(mat3(transpose(inverse(pc.model))) * inNormal);
    fragTexCoord = inTexCoord;
}
