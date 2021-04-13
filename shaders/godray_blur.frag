#version 450

layout (location = 0) in vec2 uv;

layout(set = 0, binding = 0) uniform sampler u_sampler;
layout(set = 0, binding = 1) uniform texture2D u_texture;

layout (location = 0) out vec4 colour;

layout(push_constant) uniform GodraySettings {
    float density;
    float weight;
    float decay;
    float exposure;
    vec2 uv_space_light_pos;
    uint num_samples;
};

// Adapted from https://github.com/Erkaman/glsl-godrays/blob/master/index.glsl
void main() {
    vec3 output_colour = vec3(0.0);

	vec2 delta_uv = vec2(uv - uv_space_light_pos)
        * (1.0 /  float(num_samples)) * density;

	float illumination_decay = 1.0;

    vec2 sample_uv = uv;

	for(uint i = 0; i < num_samples; i += 1){
		sample_uv -= delta_uv;
		vec3 contribution = textureLod(sampler2D(u_texture, u_sampler), sample_uv, 0).rgb
            * illumination_decay * weight;
		output_colour += contribution;
		illumination_decay *= decay;
	}

	output_colour *= exposure;

    colour = vec4(output_colour, 1.0);
}
