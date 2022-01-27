# horizon
A 2d game framework / engine.

Planned features:
- Will include in-game editing and debugging tools
- Will hopefully be less meme-y once we reach somewhere close to functionality

## Structure
Horizon is made of several parts:
- horizon_engine: the main game engine
- horizon_horse: a custom format for data serialization
- rkpk: an atlas generation and compositing library

We have a `start` (`start.bat` on windows) script to help with starting a binary project from the root folder, simply run `./start horizon` (substituting `horizon` for the crate you want to run).

## Other things

- [Code style information](./style_guide.md)
- Repo is public now because we honestly do not care about other people seeing our progress