#ifndef _TRIANGLE_DATA_H
#define _TRIANGLE_DATA_H

struct TriangleData {
    vec3 tangeant;
    vec3 bitangeant;
    vec3 normal;
    vec3 tex_orig;
    vec3 tex_u;
    vec3 tex_v;
    uint material;
};

#endif // _TRIANGLE_DATA_H
