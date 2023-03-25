struct CustomMaterial {
    color: vec4<f32>,
    //n: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
// @group(1) @binding(1)
// var base_color_texture: texture_2d<f32>;
// @group(1) @binding(2)
// var base_color_sampler: sampler;

// https://www.shadertoy.com/view/XtBfzz
fn gridTexture(p: vec2<f32> ) -> f32
{
    var N: f32 = 10.0;
    var i: vec2<f32> = step( fract(p), vec2<f32>(1.0/N) );
    return (1.0-i.x)*(1.0-i.y);   // grid (N=10)        
    
    // other possible patterns are these
    //return 1.0-i.x*i.y;           // squares (N=4)
    //return 1.0-i.x-i.y+2.0*i.x*i.y; // checker (N=2)
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {

    var p = gridTexture(uv * 10.0);
    // return vec4(col,1.0);
    var col = material.color * (1. - p);
    return vec4(col);
}
