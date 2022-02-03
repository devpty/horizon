# horizon and horizon projects style guide

we follow the official rust style guide except with a few modifications:

- use tabs for initial indentation, but spaces after the first character
	- prefer two-character wide tabs
- 80 columns maximum line length
- don't align multi-line arguments, have them all indented one level
- aligning things after the first character is preferred if it increases readability
- doc-strings don't have to contain full sentences unless the item being targeted is public and in a release
- ~~prefer `use`ing modules instead of individual items~~ this is outdated and needs changing in the existing codebase

for now this formatting must be considered manually, eventually a rustfmt configuration file will be provided
