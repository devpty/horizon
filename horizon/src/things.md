# vertex / instance data

- every rect instance is stored as a position, texture, and rotation, anchor
- they're all instances of the following triangles:
	- triangle 1:
		- vertex 1: `(pos.x,         pos.y,         tex.x,         tex.y)`
		- vertex 2: `(pos.x + pos.w, pos.y,         tex.x + tex.w, tex.y)`
		- vertex 3: `(pos.x,         pos.y + pos.h, tex.x,         tex.y + tex.h)`
	- triangle 2:
		- vertex 1: `(pos.x + pos.w, pos.y,         tex.x + tex.w, tex.y)`
		- vertex 2: `(pos.x + pos.w, pos.y + pos.h, tex.x + tex.w, tex.y + tex.h)`
		- vertex 3: `(pos.x,         pos.y + pos.h, tex.x,         tex.y + tex.h)`
	- rotated around `(pos.x + pos.w / 2, pos.y + pos.h / 2)`
	- alteratively 1.1, 2.1, 2.2, 1.3 if we use strip mode
- pos is in 0 - screen width, 0 - screen height coordinates, in pixels
- texture is in a similar system but with the atlas size instead

- vertex shader:
	- input: vertex id (0-5 or 0-2), instance data, view matrix
	- local space rect = `(pos.w * -anchor.x, pos.h * -anchor.x, pos.w, pos.h)`
	- rotate rect by the `rot`
	- translate by `(pos.w, pos.h)`
	- uv coord is based on the vertex id and tex and is scaled by the atlas size
	- scale pos by view matrix
	- return pos and uv

- fragment shader:
	- renders texture
	- brighten / dim based on distance to lights?
		- sum of `1 / distance²` for all lights within range
		- how to iterate through lights?
			- we could implement this as a screen-space shader rather than part of the frag shader

- init process:
	- get vertexes

	- create vertex buffer
		- stores the vertex data to upload to the gpu

	- create instance buffer
		- stores instances and their data

	- create bind group layout
		- defines shader passes?

	- create pipeline layout
		- container for bind group layout

	- create texture and associated info
		- texture might change when new areas are loaded so this step should be repeatable

	- create view matrix
		- ortho for screen size; account for centering / integer scale here
		- TODO: figure out what steps depend on screen size, since this isn't the only one

	- create uniform buffer
		- basically just stores the view matrix
		- will also be used to store other global info if i need it

	- create bind group
		- uses the bind group layout and links it with the data
			- for vert it adds the uniform buffer, for frag it adds the texture view

	- create / compile shader
		- self explanatory

	- create vertex buffers
		- defines how the vertex information is stored
		- TODO: something similar probably needs to be done for instancing

	- create pipeline
		- create debug wire pipeline if available
		- combines the shader and associated information

	- return info
		- yay!
