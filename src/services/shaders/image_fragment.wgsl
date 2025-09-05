// Fragment shader for GPU-accelerated image processing
// MediaOrganizer - wgpu 0.17 compatible fragment shader with enhancement filters

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct FragmentInput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    // Sample the base texture
    let base_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Apply basic color enhancement
    // This could be extended with more complex image processing operations
    let enhanced_color = enhance_colors(base_color);
    
    return enhanced_color;
}

// Color enhancement function
fn enhance_colors(color: vec4<f32>) -> vec4<f32> {
    // Slightly increase contrast and saturation for better preview quality
    let contrast = 1.1;
    let saturation = 1.05;
    
    // Apply contrast adjustment
    var adjusted = (color.rgb - 0.5) * contrast + 0.5;
    
    // Apply saturation adjustment
    let luminance = dot(adjusted, vec3<f32>(0.299, 0.587, 0.114));
    adjusted = mix(vec3<f32>(luminance), adjusted, saturation);
    
    // Ensure values stay within valid range
    adjusted = clamp(adjusted, vec3<f32>(0.0), vec3<f32>(1.0));
    
    return vec4<f32>(adjusted, color.a);
}

// Additional utility function for gamma correction
fn gamma_correct(color: vec3<f32>, gamma: f32) -> vec3<f32> {
    return pow(color, vec3<f32>(1.0 / gamma));
}