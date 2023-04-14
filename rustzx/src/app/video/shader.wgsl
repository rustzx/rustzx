/// Global shader params
struct Globals {
    palette: array<vec4<f32>, 16>,
    content_aspect_ratio: f32,
    screen_aspect_ratio: f32,
    texture_atlas_size: vec2<f32>,
};

/// Vertex shader output
struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> globals: Globals;


@group(0)
@binding(1)
var atlas_texture: texture_2d<u32>;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    // ### Inputs:
    // -> Screen width (SW) -> uniform value
    // -> Screen height (SH) -> uniform value
    // -> Content aspect ratio (CAR) -> 4:3
    // -> Screen aspect ratio (SW / SH = SAR) -> calculated in shader
    // -> Vertices in [1.0, 1.0] coordinates, y points DOWN
    // ### Outputs:
    // -> Scale factor 2D vector (SF)
    //     -> X scale in screen space (-1.0, 1.0)
    //     -> Y scale in screen space (-1.0, 1.0), y points UP
    // Output values are calculated differently depending on the aspect ratios of
    // the content and the screen.
    // ### Landscape: (CAR <= SAR)
    //   -> SF.x scale is CAR/SAR
    //   -> SF.y scale is 1.0
    // ### Portrait: (car > sar)
    //   -> SF.x scale is 1.0
    //   -> SF.y scale is SAR/CAR

    // Fit notmalized coordinates [0; 1] to [-1, 1] WegbGL screen space and flip Y axis
    let screen_space_position = position * vec2<f32>(2.0, -2.0) - vec2<f32>(1.0, -1.0);

    let sf = select(
        vec2<f32>(1.0, globals.screen_aspect_ratio / globals.content_aspect_ratio), // false
        vec2<f32>(globals.content_aspect_ratio / globals.screen_aspect_ratio, 1.0), // true
        globals.content_aspect_ratio <= globals.screen_aspect_ratio,                // condition
    );

    let position = vec4<f32>(screen_space_position * sf, 1.0, 1.0);

    return VertexOutput(tex_coord, position);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // Read indexed color value from 8-bit single-channel texture
    let texel_coords = vec2<u32>(vertex.tex_coord * globals.texture_atlas_size);
    let index = textureLoad(atlas_texture, texel_coords, 0).r;
    // Obtain color from the palette based on the color index
    return globals.palette[index];
}
