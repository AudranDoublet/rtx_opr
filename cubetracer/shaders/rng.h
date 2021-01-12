#ifndef  _RNG_H_
#define  _RNG_H_

#extension GL_GOOGLE_include_directive : enable
#include "constants.h"

// source: https://gist.github.com/patriciogonzalezvivo/670c22f3966e662d2f83
float rand(vec2 n) { 
	return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
}

float noise(vec2 p){
	vec2 ip = floor(p);
	vec2 u = fract(p);
	u = u*u*(3.0-2.0*u);
	
	float res = mix(
		mix(rand(ip),rand(ip+vec2(1.0,0.0)),u.x),
		mix(rand(ip+vec2(0.0,1.0)),rand(ip+vec2(1.0,1.0)),u.x),u.y);
	return res*res;
}


vec3 sample_cos_hemisphere(vec2 uv)
{
    float theta = 2.0 * C_PI * uv.x;
    float r = sqrt(uv.y);

    vec2 disk = vec2(cos(theta), sin(theta)) * r;
    return vec3(disk.x, sqrt(max(0.0, 1.0 - dot(disk, disk))), disk.y);
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

#endif // _RNG_H_
