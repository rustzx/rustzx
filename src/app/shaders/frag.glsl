#version 110

varying vec2 v_tex_coord;
uniform sampler2D tex_screen;
uniform sampler2D tex_border;

void main() {
    vec4 scr = texture2D(tex_screen, v_tex_coord);
    gl_FragColor =  scr + texture2D(tex_border, v_tex_coord) * (1.0 - scr.a);
}
