#version 430 core

uniform sampler2D uni_tex;
in vec2 texCoord;
out vec4 out_color;

void main() {
    out_color = texture(uni_tex, texCoord);
}
