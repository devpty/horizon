# horizon
A 2d game framework / engine.

Planned features:
- Will include in-engine editing and debugging tools
- Will hopefully be less meme-y once we reach somewhere close to functionality

## Structure
Horizon is made of several parts:
- horizon: the main game engine
- rkpk: an atlas generation and compositing library
- asset: asset packing thing

We have a `start` (`start.bat` on windows) script to help with starting a binary project from the root folder, simply run `./start horizon` (substituting `horizon` for the crate you want to run).

## Other things

- [Code style information](./style_guide.md)
- Repo is public now because we honestly do not care about other people seeing our progress
- **You need to be using a nightly version of rust** because we use the following unstable features:
  - label_break_value
