#version 450

layout(push_constant) uniform SkyboxConstants {
    mat4 invProjection;
    mat4 cameraRotation;
    mat4 skyboxRotation;  // rotates the sky independently (day/night)
} pushConstants;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec3 worldDirection;

void main() {
    gl_Position = vec4(inPosition.xy, 1.0, 1.0);

    // Unproject NDC to view-space ray
    vec4 viewPos = pushConstants.invProjection * vec4(inPosition.xy, 1.0, 1.0);
    vec3 viewDir = normalize(viewPos.xyz / viewPos.w);

    // Camera rotation → world space, then apply skybox rotation on top
    vec3 worldDir = (pushConstants.cameraRotation * vec4(viewDir, 0.0)).xyz;
    worldDirection = (pushConstants.skyboxRotation * vec4(worldDir, 0.0)).xyz;
}
