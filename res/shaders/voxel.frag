// Fragment Shader
#version 450

layout(location = 0) in vec3 fragNormal;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 baseColor = vec3(0.4, 0.8, 0.3); // Green grass/dirt color
    
    // Light direction (normalized)
    vec3 lightDir = normalize(vec3(0.5, 1.0, 0.3));
    
    // Calculate diffuse lighting
    float diffuse = max(dot(fragNormal, lightDir), 0.0);
    
    // Ambient lighting
    float ambient = 0.4;
    
    // Directional shading factors for each face
    float shadeFactor = 1.0;
    
    // Top face: brightest
    if (fragNormal.y > 0.9) {
        shadeFactor = 1.0;
    }
    // Bottom face: darkest
    else if (fragNormal.y < -0.9) {
        shadeFactor = 0.5;
    }
    // Side faces: medium brightness
    else if (abs(fragNormal.x) > 0.9) {
        shadeFactor = 0.75; // Left/Right
    }
    else if (abs(fragNormal.z) > 0.9) {
        shadeFactor = 0.65; // Front/Back
    }
    
    // Combine ambient and diffuse with directional shading
    float lighting = ambient + diffuse * 0.6;
    vec3 finalColor = baseColor * lighting * shadeFactor;
    
    outColor = vec4(finalColor, 1.0);
}
