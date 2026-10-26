# Feature 28: Navigable Galaxy Map

## Overview
The GUI galaxy chart is one continuous map you can move around and zoom, from the keyboard or the mouse, instead of two fixed views.

## Status
In progress, in slices: #266 pan and zoom, #267 recentre and label levels, #268 choose stars with the keyboard.

## Version 1 Scope
- Arrow keys pan the map; holding a key keeps panning
- `+` and `-` (also `=` and the numpad), the mouse wheel and the on-screen buttons zoom; the wheel zooms around the pointer
- Zoom runs from the whole galaxy (all 256 stars) in view up to 8 times the default, which fits the jump range
- The jump-range circle, the route line and the star markers scale with the view
- Opening the screen resets the view: centred on the commander's star at the default zoom, with the map focused so the keys work at once
- Clicking a star still selects it
- Later slices: `Home` and a Centre on me button, star names that appear as you zoom in, and Shift+arrow to choose stars

## Not in v1
- Dragging the map with the mouse to pan (maybe later)
- Remembering the view between visits to the screen

## Notes
- GUI only; the Rust core is unchanged
