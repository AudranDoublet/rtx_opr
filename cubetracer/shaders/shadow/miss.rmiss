#version 460
#extension GL_NV_ray_tracing : require

layout(location = 0) rayPayloadInNV bool shadowed;

void main() {
    shadowed = false;
}
