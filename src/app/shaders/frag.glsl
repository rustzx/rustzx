#version 110

varying vec2 v_tex_coord;
uniform sampler2D tex;

void main() {
    vec4 tex_sample = texture2D(tex, v_tex_coord);
    gl_FragColor =  tex_sample;
}
