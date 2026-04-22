#version 450

layout(location = 0) in uint data_lo;
layout(location = 1) in uint data_hi;

layout(location = 0) out vec2 fragUV;
layout(location = 1) out flat uint fragTexId;
layout(location = 2) out flat uint fragAtlasTiles;

layout(push_constant) uniform Push {
    mat4 proj_view;   // 64 bytes
    mat4 model;       // 64 bytes
    mat4 chunk_pos;   // 64 bytes — or use a vec4 + padding instead
    uint atlas_tiles; // 4 bytes
} pc;

void main() {
    uint x    = (data_lo >> 0u)  & 0x3Fu;
    uint y    = (data_lo >> 6u)  & 0x3Fu;
    uint z    = (data_lo >> 12u) & 0x3Fu;
    uint face = (data_lo >> 18u) & 0x7u;
    uint u    = (data_lo >> 21u) & 0x3Fu;
    uint v    = (data_lo >> 27u) & 0x1Fu;
    uint tex  = (data_lo >> 32u) | (data_hi & 0x1u);


    fragUV = vec2(float(u), float(v));
    fragTexId = tex;
    fragAtlasTiles = pc.atlas_tiles;

    gl_Position = pc.proj_view * vec4(float(x), float(y), float(z), 1.0);
}
