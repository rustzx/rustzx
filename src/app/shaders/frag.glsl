#version 140

vec3 palette[16] = vec3[](
    vec3(0.0, 0.0, 0.0),
    vec3(0.0, 0.0, 0.5),
    vec3(0.5, 0.0, 0.0),
    vec3(0.5, 0.0, 0.5),
    vec3(0.0, 0.5, 0.0),
    vec3(0.0, 0.5, 0.5),
    vec3(0.5, 0.5, 0.0),
    vec3(0.5, 0.5, 0.5),

    vec3(0.0, 0.0, 0.0),
    vec3(0.0, 0.0, 1.0),
    vec3(1.0, 0.0, 0.0),
    vec3(1.0, 0.0, 1.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 1.0),
    vec3(1.0, 1.0, 0.0),
    vec3(1.0, 1.0, 1.0)
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
    color = vec4(palette[paper], 1.0) * float(1 - int(tex_col.g) / 15) +
        vec4(palette[ink], 1.0)  * float(int(tex_col.g) / 15);
}
