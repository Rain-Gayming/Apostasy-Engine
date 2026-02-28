#version 450
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;


layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
    vec4 offset;
    vec4 rotation;
    vec4 scale;
} pc;

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec2 fragTexCoord;

vec3 applyQuaternion(vec4 q, vec3 v) {
    vec3 qv = vec3(q.x, q.y, q.z);
    return v + 2.0 * cross(qv, cross(qv, v) + q.w * v);
}


void main() {
 vec3 scale = vec3(pc.scale.x, pc.scale.y, pc.scale.z);
    vec3 scaledPosition = inPosition * scale;                
    vec3 rotatedPosition = applyQuaternion(pc.rotation, scaledPosition);
    vec3 offset = vec3(pc.offset.x, pc.offset.y, pc.offset.z);

    gl_Position = pc.mvp * vec4(rotatedPosition + offset, 1.0);

    vec3 rotatedNormal = applyQuaternion(pc.rotation, inNormal / scale);
    fragNormal = normalize(mat3(transpose(inverse(pc.model))) * rotatedNormal);
    fragTexCoord = inTexCoord;
}
