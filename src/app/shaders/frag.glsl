#version 140

const vec4 palette[16] = vec4[](
    vec4(0.0, 0.0, 0.0, 1.0),
    vec4(0.0, 0.0, 0.5, 1.0),
    vec4(0.5, 0.0, 0.0, 1.0),
    vec4(0.5, 0.0, 0.5, 1.0),
    vec4(0.0, 0.5, 0.0, 1.0),
    vec4(0.0, 0.5, 0.5, 1.0),
    vec4(0.5, 0.5, 0.0, 1.0),
    vec4(0.5, 0.5, 0.5, 1.0),

    vec4(0.0, 0.0, 0.0, 1.0),
    vec4(0.0, 0.0, 1.0, 1.0),
    vec4(1.0, 0.0, 0.0, 1.0),
    vec4(1.0, 0.0, 1.0, 1.0),
    vec4(0.0, 1.0, 0.0, 1.0),
    vec4(0.0, 1.0, 1.0, 1.0),
    vec4(1.0, 1.0, 0.0, 1.0),
    vec4(1.0, 1.0, 1.0, 1.0)
);

out vec4 color;
in vec2 v_tex_coord;

uniform usampler2D tex;
uniform bool blink;

void main() {
    uvec4 tex_col = texture(tex, v_tex_coord);
    uint paper = tex_col.b;
    uint ink = tex_col.a;
    if ((tex_col.r != uint(0)) && (blink)) {
        uint tmp = paper;
        paper = ink;
        ink = tmp;

    }
    if (tex_col.g == uint(0)) {
        color = palette[paper];
    } else {
        color = palette[ink];
    }
}
