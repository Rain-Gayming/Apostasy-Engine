#version 450

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec3 fragNormal;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
} pc;

vec3 positions[24] = vec3[](
    // Front face (Z+)
    vec3(-0.5, -0.5,  0.5),
    vec3( 0.5, -0.5,  0.5),
    vec3( 0.5,  0.5,  0.5),
    vec3(-0.5,  0.5,  0.5),
    // Back face (Z-)
    vec3( 0.5, -0.5, -0.5),
    vec3(-0.5, -0.5, -0.5),
    vec3(-0.5,  0.5, -0.5),
    vec3( 0.5,  0.5, -0.5),
    // Right face (X+)
    vec3( 0.5, -0.5, -0.5),
    vec3( 0.5, -0.5,  0.5),
    vec3( 0.5,  0.5,  0.5),
    vec3( 0.5,  0.5, -0.5),
    // Left face (X-)
    vec3(-0.5, -0.5,  0.5),
    vec3(-0.5, -0.5, -0.5),
    vec3(-0.5,  0.5, -0.5),
    vec3(-0.5,  0.5,  0.5),
    // Top face (Y+)
    vec3(-0.5,  0.5, -0.5),
    vec3( 0.5,  0.5, -0.5),
    vec3( 0.5,  0.5,  0.5),
    vec3(-0.5,  0.5,  0.5),
    // Bottom face (Y-)
    vec3(-0.5, -0.5,  0.5),
    vec3( 0.5, -0.5,  0.5),
    vec3( 0.5, -0.5, -0.5),
    vec3(-0.5, -0.5, -0.5)
);

vec3 normals[24] = vec3[](
    vec3(0.0, 0.0,  1.0), vec3(0.0, 0.0,  1.0), vec3(0.0, 0.0,  1.0), vec3(0.0, 0.0,  1.0),
    vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, -1.0),
    vec3(1.0, 0.0,  0.0), vec3(1.0, 0.0,  0.0), vec3(1.0, 0.0,  0.0), vec3(1.0, 0.0,  0.0),
    vec3(-1.0, 0.0,  0.0), vec3(-1.0, 0.0,  0.0), vec3(-1.0, 0.0,  0.0), vec3(-1.0, 0.0,  0.0),
    vec3(0.0, 1.0,  0.0), vec3(0.0, 1.0,  0.0), vec3(0.0, 1.0,  0.0), vec3(0.0, 1.0,  0.0),
    vec3(0.0, -1.0,  0.0), vec3(0.0, -1.0,  0.0), vec3(0.0, -1.0,  0.0), vec3(0.0, -1.0,  0.0)
);

vec3 colors[6] = vec3[](
    // red
    vec3(1.0, 0.0, 0.0),
    // cyan
    vec3(0.0, 1.0, 1.0),
    // green
     vec3(0.0, 1.0, 0.0),
    // magenta
     vec3(1.0, 0.0, 1.0),
    // yellow
    vec3(1.0, 1.0, 0.0), 
    // blue
    vec3(0.0, 0.0, 1.0)
);

int indices[36] = int[](
    3, 2, 1, 3, 1, 0,
    5, 4, 7, 5, 7, 6,
    11, 10, 8, 10, 9, 8,
    13, 14, 12, 14, 15, 12,
    17, 18, 16, 18, 19, 16,
    21, 22, 20, 22, 23, 20
);

void main() {
    int idx = indices[gl_VertexIndex];
   
    vec3 pos = positions[idx];
    vec3 normal = normals[idx];
    
    vec4 worldPos = pc.model * vec4(pos, 1.0);
    vec4 clipPos = pc.mvp * vec4(pos, 1.0);
    gl_Position = clipPos;
    
    fragColor = colors[idx / 6];
    fragNormal = normalize((pc.model * vec4(normal, 0.0)).xyz);
}
