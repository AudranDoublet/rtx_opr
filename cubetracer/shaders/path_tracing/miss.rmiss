#version 460
#extension GL_NV_ray_tracing : require
#extension GL_GOOGLE_include_directive : enable

#include "../payload.h"

layout(location = 0) rayPayloadInNV InitialPayload payload;

void main() {
    payload.hit = false;
}
