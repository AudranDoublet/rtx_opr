#version 460
#extension GL_NV_ray_tracing : require
#extension GL_GOOGLE_include_directive : enable

#include "../payloads.h"

layout(location = PAYLOAD_IDX_GENERAL) rayPayloadInNV InitialPayload PAYLOAD_GENERAL;

void main() {
    PAYLOAD_GENERAL.hit = false;
}
