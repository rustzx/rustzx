#version 140

out vec4 color;

in vec2 v_tex_coord;
uniform sampler2D tex_screen;
uniform sampler2D tex_border;

void main() {
    vec4 scr = texture(tex_screen, v_tex_coord);
    color =  scr + texture(tex_border, v_tex_coord) * (1.0f - scr.a);
}
