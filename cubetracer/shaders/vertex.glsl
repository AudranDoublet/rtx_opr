#version 430 core

in vec2 in_pos;
out vec2 texCoord;

void main() {
    // normalizing coordinates into texture uv range [0, 1]
    texCoord = in_pos * 0.5 + 0.5;

    gl_Position = vec4(in_pos, 0.0, 1.0);
}
