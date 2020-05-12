#version 430 core

layout (local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

uniform float roll;
layout (binding = 0, rgba32f) uniform image2D out_frame;

void main() {
    ivec2 pixel = ivec2(gl_GlobalInvocationID.xy);
    float localCoef = length(vec2(ivec2(gl_LocalInvocationID.xy) - 8)/8.0);
    float globalCoef = sin(float(gl_WorkGroupID.x + gl_WorkGroupID.y)*0.1+roll)*0.5;
    vec4 color = vec4(vec3(1.0-globalCoef*localCoef), 1.0);
    imageStore(out_frame, pixel, color);
}
