#version 450
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in uint vertexData;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
} pc;

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec2 fragTexCoord;

void main() {
    if (vertexData == 0) {
        gl_Position = pc.mvp * vec4(inPosition, 1.0);
        fragNormal = normalize(mat3(transpose(inverse(pc.model))) * inNormal);
        fragTexCoord = inTexCoord;
    }else{
        int posZ = (data >> 12) & 63;
        int posY = (data >> 6) & 63;
        int posX = data & 63;
        vec3 pos = vec3(posX, posY, posZ);
        gl_Position = pc.mvp * vec4(pos, 1.0);
        fragNormal = normalize(mat3(transpose(inverse(pc.model))) * inNormal);
        fragTexCoord = inTexCoord;

    }
    
}
