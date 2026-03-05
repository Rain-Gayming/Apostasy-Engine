#version 450
layout(set = 0, binding = 1) uniform sampler2D texSampler;


layout(push_constant) uniform PushConstants {
    mat4 mvp;        // 0
    mat4 model;      // 64
    vec4 offset;     // 128
    vec4 rotation;   // 144
    vec4 scale;      // 160
    vec4 base_color; // 176
    float metallic;  // 192
    float roughness; // 196
    float _pad0;     // 200
    float _pad1;     // 204
    vec3 emissive;   // 208
    float _pad2;     // 220
    vec4 light_pos;  // 224
} pc;

layout(location = 0) out vec4 outColor;
layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in vec3 fragPos;   


void main() {
    vec3 normal = normalize(fragNormal);
    vec3 lightDir = normalize(pc.light_pos.xyz - fragPos);
    float diff = clamp(dot(normal, lightDir), 0.0, 1.0);
    float ambient = 0.2;
    float brightness = ambient + (0.7 - ambient) * diff * pc.light_pos.w;
    vec4 texColor = texture(texSampler, fragTexCoord);
    vec3 color = texColor.rgb * pc.base_color.rgb * brightness + pc.emissive;
    outColor = vec4(color, 1.0);
}
