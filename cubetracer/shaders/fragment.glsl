#version 430 core

uniform sampler2D uni_tex;

in vec2 texCoord;
out vec4 out_color;

#ifdef GL_ES
precision mediump float;
#endif

#define SIGMA_R 0.1
#define EFF_SIGMA (1.0 / (2.0*SIGMA_R*SIGMA_R))

#define SAMPLE_COUNT 8

/*
Python function used to generate this values:
def values(sigma):
    import math

    sigma = 2 * sigma * sigma

    for i in [-1, 0, 1]:
        for j in [-1, 0, 1]:
            if (i, j) != (0, 0):
                print(math.exp(-(i*i + j*j) / sigma))
*/
const float[SAMPLE_COUNT] distance_coeffs = {
    0.9900498337491681,
    0.9950124791926823,
    0.9900498337491681,
    0.9950124791926823,
    0.9950124791926823,
    0.9900498337491681,
    0.9950124791926823,
    0.9900498337491681,
};

const vec2[SAMPLE_COUNT] samples = {
    vec2(-1.0, -1.0),
    vec2(0.0, -1.0),
    vec2(1.0, -1.0),

    vec2(-1.0, 0.0),
    vec2(1.0, 0.0),

    vec2(-1.0, 1.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
};

float color_coefficient(const vec4 diff) {
    return exp(-dot(diff, diff) * EFF_SIGMA);
}

void main() {
    vec4 center_color = texture(uni_tex, texCoord);

    ivec2 isize = textureSize(uni_tex, 0);
    vec2 size = 1.0 / vec2(float(isize.x), float(isize.y));

    vec4 result = center_color;
    float total_weight = 1.0;

    for (int i = 0; i < SAMPLE_COUNT; i++)
    {
        vec4 color = texture(uni_tex, texCoord + samples[i] * size);
        float weight = color_coefficient(center_color - color) * distance_coeffs[i];

        total_weight += weight;
        result += color * weight;
    }

    out_color = result / total_weight;
}
