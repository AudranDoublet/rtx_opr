#version 460
#extension GL_NV_ray_tracing : require
#extension GL_GOOGLE_include_directive : enable

#include "../constants.h"
#include "../payloads.h"

layout(location = PAYLOAD_IDX_GENERAL) rayPayloadInNV InitialPayload PAYLOAD_GENERAL;

void main() {
    if (gl_WorldRayDirectionNV.y < 0)
        PAYLOAD_GENERAL.illumination = vec3(1.0, 0.0, 0.0);
    else
        PAYLOAD_GENERAL.illumination = vec3(0.0, 1.0, 0.0);

    PAYLOAD_GENERAL.hit = false;
}
