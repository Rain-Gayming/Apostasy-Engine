#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragNormal;
layout(location = 0) out vec4 outColor;

void main() {
    // Basic directional lighting
    vec3 lightDir = normalize(vec3(0.5, 1.0, 1.0));
    float brightness = max(0.3, dot(fragNormal, lightDir));
    
    vec3 lit = fragColor * brightness;
    outColor = vec4(lit, 1.0);
}
