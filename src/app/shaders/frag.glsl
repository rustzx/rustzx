#version 140

out vec4 color;
in vec2 v_tex_coord;

uniform sampler2D tex;

void main() {
    color = texture(tex, v_tex_coord);
}
