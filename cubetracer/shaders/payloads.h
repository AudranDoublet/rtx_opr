#ifndef  _PAYLOADS_H_
#define  _PAYLOADS_H_

struct InitialPayload {
    bool hit;
    float distance;
    vec3 normal;
    vec3 illumination;
    vec3 hit_position;
    vec3 mer;
};


#endif // _PAYLOADS_H
