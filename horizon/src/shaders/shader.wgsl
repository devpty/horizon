// vert shader
struct UniIn {
	offset: vec2<f32>;
	size: vec2<f32>;
	atlas_res: vec2<f32>;
};
[[group(0), binding(0)]]
var t_atlas: texture_2d<f32>;
[[group(0), binding(1)]]
var s_atlas: sampler;
[[group(1), binding(0)]]
var<uniform> uni: UniIn;
struct VertIn {
	[[location(0)]] pos: vec2<f32>;
	[[location(1)]] uv:  vec2<f32>;
	[[location(2)]] col: vec4<f32>;
};
struct VertOut {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] color: vec4<f32>;
	[[location(1)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vert(
	vert: VertIn,
) -> VertOut {
	var out: VertOut;
	out.color = vert.col;
	out.tex_coords = vert.uv / uni.atlas_res;
	// f32(pos + offset) / f32(size) - vec2<f32>(1.0, 1.0)
	let pos2d = (vert.pos + uni.offset) * vec2<f32>(uni.size);
	out.clip_position = vec4<f32>((2.0 * pos2d - 1.0) * vec2<f32>(1.0, -1.0), 0.5, 1.0);
	return out;
}

// frag shader
[[stage(fragment)]]
fn frag(
	in: VertOut
) -> [[location(0)]] vec4<f32> {
	return textureSample(t_atlas, s_atlas, in.tex_coords) * in.color;
}
