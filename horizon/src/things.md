# Render Data Structure
<sup>(needs reworking in source code)</sup>

## In Memory
- a new struct, `RenderContext` is used to store data
- `RenderContext` has a `Vec<VertexData>` and a `Vec<u16>`, these are synced up to the GPU's vertex and index buffers on render
- polygon clipping is done on the CPU instead of the GPU

## Structs and stuff
- `struct Node` - kinda like a unity GameObject, stores:
	- a transform
		- position (x, y)
		- rotation (angle)
		- scale (x, y)
		- clip rect: optional rectangle
	- a weak reference to the parent
	- a list of children `Node`s
	- a list of children `Component`s (most likely in some kind of wrapper struct)
- `trait Component` - kinda like a unity MonoBehaviour, with the following:
	- an optional `render` method
	- an optional `update` method

## Control Flow
- `State::render` is called by the event loop
	- that calls `State::update`
		- which calls the root node's `update` method
			- which updates the state
			- and recurses down the node tree
	- it calls the root node's `render` method with a cleared `RenderContext` and a full-screen clip rectangle
		- which pushes polygons using `RenderContext::push_tri()`, `RenderContext::push_rect()`, etc. 
		- and recurses down the node tree