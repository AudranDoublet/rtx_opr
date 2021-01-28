#ifndef  _PAYLOADS_H_
#define  _PAYLOADS_H_

struct InitialPayload {
    bool hit;
    float distance;
    uint material;
    vec3 normal;
    vec3 illumination;
    vec3 hit_position;
    vec3 mer;
};

#define PAYLOAD_IDX_GENERAL  0
#define PAYLOAD_IDX_SHADOWED 1

#endif // _PAYLOADS_H
