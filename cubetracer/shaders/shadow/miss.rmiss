#version 460
#extension GL_NV_ray_tracing : require
#extension GL_GOOGLE_include_directive : enable

#include "../payloads.h"

layout(location = PAYLOAD_IDX_SHADOWED) rayPayloadInNV bool PAYLOAD_SHADOWED;

void main() {
    PAYLOAD_SHADOWED = false;
}
