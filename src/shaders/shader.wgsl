// vert shader
struct VertIn {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] color: vec3<f32>;
};
struct VertOut {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] color: vec3<f32>;
};

[[stage(vertex)]]
fn vert(
	model: VertIn
) -> VertOut {
	var out: VertOut;
	out.color = model.color;
	out.clip_position = vec4<f32>(model.position, 1.0);
	return out;
}

// frag shader
[[stage(fragment)]]
fn frag(
	in: VertOut
) -> [[location(0)]] vec4<f32> {
	return vec4<f32>(in.color, 1.0);
}
