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
} pc;

layout(location = 0) out vec4 outColor;
layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragTexCoord;

void main() {
    float brightness = dot(fragNormal, vec3(-1.0, 0.5, 0.9));
    vec3 regular_color = vec3(1.0, 1.0, 1.0);
    vec3 dark_color = vec3(0.5, 0.5, 0.5);
    vec3 lighting = mix( regular_color, dark_color, brightness);

    vec4 texColor = texture(texSampler, fragTexCoord);
    vec3 base = pc.base_color.rgb;
    vec3 emiss = pc.emissive;
    float alpha = pc.base_color.a;

    vec3 color = texColor.rgb * base * lighting + emiss;
    outColor = vec4(color, texColor.a * alpha);
}
