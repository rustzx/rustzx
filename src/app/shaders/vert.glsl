#version 110

uniform mat4 matrix;

attribute vec2 position;
attribute vec2 tex_coord;

varying vec2 v_tex_coord;

void main() {
    v_tex_coord = tex_coord;
    gl_Position = matrix * vec4(position, 0.0, 1.0);
}
