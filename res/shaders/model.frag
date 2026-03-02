#version 450
layout(set = 0, binding = 1) uniform sampler2D texSampler;
layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
    vec4 offset;
    vec4 rotation;
    vec4 scale;
    vec4 base_color;
    float metallic;
    float roughness;
    vec3 emissive;
    vec4 light_pos;
} pc;
layout(location = 0) out vec4 outColor;
layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in vec3 fragPos;   
void main() {
    vec3 normal = normalize(fragNormal);
    vec3 lightDir = normalize(pc.light_pos.xyz - fragPos);
    float diff = clamp(dot(normal, lightDir), 0.0, 1.0);

    // Ambient + diffuse, actually affects brightness
    float ambient = 0.2;
    float brightness = ambient + (1.0 - ambient) * diff * pc.light_pos.w;

    vec4 texColor = texture(texSampler, fragTexCoord);
    vec3 base = pc.base_color.rgb;
    vec3 emiss = pc.emissive.rgb;

    vec3 color = texColor.rgb * base * brightness + emiss;
    outColor = vec4(color, texColor.a * pc.base_color.a);
}
