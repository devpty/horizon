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
struct InstIn {
	[[location(1)]] pos: vec2<f32>;
	[[location(2)]] origin: vec2<f32>;
	[[location(3)]] uv: vec4<u32>;
	[[location(4)]] color: vec4<f32>;
	[[location(5)]] size_rot_flags: vec4<u32>;
};
struct VertIn {
	[[location(0)]] pos: vec2<u32>;
};
struct VertOut {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] color: vec4<f32>;
	[[location(1)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vert(
	vert: VertIn,
	in: InstIn,
) -> VertOut {
	var out: VertOut;
	out.color = in.color;
	let offset = (vec2<f32>(vert.pos) - in.origin) * vec2<f32>(in.size_rot_flags.xy);
	out.tex_coords = vec2<f32>(vert.pos * in.uv.zw + in.uv.xy) / uni.atlas_res;
	let rot = (f32(in.size_rot_flags.z) / 65536.0) * 3.14159265358979323;
	let sinv = sin(rot);
	let cosv = cos(rot);
	let rot_offset = vec2<f32>(
		offset.x * cosv - offset.y * sinv,
		offset.y * cosv + offset.x * sinv
	);
	// f32(pos + offset) / f32(size) - vec2<f32>(1.0, 1.0)
	let pos2d = (in.pos + rot_offset + uni.offset) * vec2<f32>(uni.size);
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
