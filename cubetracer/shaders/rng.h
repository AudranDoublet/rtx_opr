#ifndef  _RNG_H_
#define  _RNG_H_

uint static_rng_seed = 0;

// source: https://nvpro-samples.github.io/vk_raytracing_tutorial/vkrt_tuto_jitter_cam.md.htm#environmentsetup/randomfunctions
// Generate a random unsigned int in [0, 2^24) given the previous RNG state
// using the Numerical Recipes linear congruential generator
uint lcg(inout uint prev)
{
    uint LCG_A = 1664525u;
    uint LCG_C = 1013904223u;
    prev       = (LCG_A * prev + LCG_C);
    return prev & 0x00FFFFFF;
}

// Generate a random float in [0, 1) given the previous RNG state
float rnd(inout uint prev)
{
    return (float(lcg(prev)) / float(0x01000000));
}


// source: https://github.com/NVIDIA/Q2RTX/blob/master/src/refresh/vkpt/shader/utils.glsl
mat3 construct_ONB_frisvad(vec3 normal)
{
    precise mat3 ret;
    ret[1] = normal;
    if(normal.z < -0.999805696f) {
        ret[0] = vec3(0.0f, -1.0f, 0.0f);
        ret[2] = vec3(-1.0f, 0.0f, 0.0f);
    }
    else {
        precise float a = 1.0f / (1.0f + normal.z);
        precise float b = -normal.x * normal.y * a;
        ret[0] = vec3(1.0f - normal.x * normal.x * a, b, -normal.x);
        ret[2] = vec3(b, 1.0f - normal.y * normal.y * a, -normal.y);
    }
    return ret;
}

vec3 sample_cos_hemisphere(vec2 uv)
{
    float theta = 2.0 * m_pi * uv.x;
    float r = sqrt(uv.y);

    vec2 disk = vec2(cos(theta), sin(theta)) * r;
    return vec3(disk.x, sqrt(max(0.0, 1.0 - dot(disk, disk))), disk.y);
}

#endif // _RNG_H_
