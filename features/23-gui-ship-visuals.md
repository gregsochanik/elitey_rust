# Feature 23: GUI Ship Visuals

## Overview
Part 2 of the GUI (Feature 15). Ships, stations and planets become data, drawn first as Elite-style wireframes and then upgraded through the same pipeline: wireframe, hidden-line, flat fills, textures and lighting. Godot's built-in 3D does the work; the Rust core and `elite-ffi` facade do not change.

## Status
Approved; sliced into issues #214 to #222 (umbrella #212). Nothing is built.

## Current state
- The Rust core, `elite-ffi` and `elite-godot` (gdext) are in place, and the menu screens, jobs, saves and the turn-based journey scene are built (Feature 15, Phase 3).
- Ship shapes are hard-coded as vertex and edge arrays inside `journey_scene.gd` (`_ship_mesh`, `_raider_mesh`) and built with `ImmediateMesh` lines. There is no data format, no faces, and no way to show a ship outside the journey.
- Line colour, glow and the starfield are set inline in the scene script. `WorldEnvironment` glow already works.
- The renderer is Godot's default (Forward+). `project.godot` does not set one.

## Principles
- The sim knows ships by **id** only (`cobra-mk-iii`, `python`, `raider`). It never holds meshes, materials or scene nodes. The view maps id to a model and a style.
- A model is **data**; a style is **how to draw it**. Changing the style must not touch models or the sim.
- No gameplay state on mesh nodes; views read the sim through `elite-ffi` as today.
- Styles live in one theme resource (colours, line widths, glow, light), not in scene scripts.

## Design

### Model data
One JSON file per model under `elite-godot/project/models/<id>.json`:

```json
{
  "id": "cobra-mk-iii",
  "vertices": [[0, 0, -3.2], [-0.9, 0.35, 0.6]],
  "edges": [[0, 1]],
  "faces": [[0, 1, 2]],
  "uvs": [[0.5, 0.0]]
}
```

- `vertices` and `edges` are required (wireframe). `faces` (triangles) arrive with the solid style. `uvs` and any texture maps arrive with the textured style.
- Edges not listed by a face stay drawn (fins, antennas); face outlines need not be repeated as edges.
- The Rust tests check every model file: indices in range, no degenerate faces, every vertex used.

### Style layer
A `VisualStyle` resource (GDScript) with a mapping `model id -> scene` for the active style:

| Style | Draws | Needs |
|---|---|---|
| `wireframe` | `ArrayMesh` with `PRIMITIVE_LINES` from the edges, unshaded, glowing | vertices, edges |
| `hidden_line` | wireframe over a black filled mesh, so far edges are hidden | faces |
| `flat` | filled triangles, flat shading, one directional light | faces |
| `textured` | `StandardMaterial3D` with albedo, normal and emission maps, lit by the system's star | faces, uvs, textures |

- Style is a setting (saved with preferences, not with the game), switchable at runtime for comparison.
- A model without the data a style needs falls back to the previous style, so ships can be upgraded one at a time.
- Lighting: a directional light coloured from the system's star class (Feature 22): a red dwarf lights ships red-orange, a white star lights them white. This is what makes `flat` and `textured` look different from system to system.

### Scenes
- A **ship viewer** (turntable, orbit camera) shows any model in any style. It appears on the Shipyard screen for the selected ship, and in a dev view that cycles every model and style.
- The **journey scene** asks the style layer for the player's ship and the enemy by id instead of calling `_ship_mesh`/`_raider_mesh`.
- Planets and stations use the same path (a sphere and a simple station model), later.

### Renderer
The project moves to the **Mobile** renderer in Phase 1 so effects port to iOS later. Godot's renderer docs list Mobile as suited to mobile and desktop and as using Vulkan, Direct3D 12 or Metal, and show glow supported in all three renderers. Not checked: whether any current effect (glow strength, bloom) looks different under Mobile; check in Phase 1 by comparing screenshots.

## Phases

### Phase 1: Models as data and the style layer
- [ ] Define the model JSON format and move the player ship and raider into `models/`, built into meshes by one loader
- [ ] `VisualStyle` resource and the id-to-scene mapping; the journey scene uses it; look unchanged in `wireframe`
- [ ] Colours, line settings, glow and starfield settings move into the style resource
- [ ] Switch to the Mobile renderer; compare screenshots of the journey scene before and after
- [ ] Rust test validates every model file

### Phase 2: Ship models and the viewer
- [ ] Wireframe models for every ship in `ships.json` (Cobra Mk III, Mk IV, Python, Anaconda), the raider, and a station
- [ ] Ship viewer scene with turntable and orbit camera; shown on the Shipyard screen
- [ ] A dev view cycling every model and style

### Phase 3: Solid style
- [ ] Add faces to the models; `hidden_line` and `flat` styles
- [ ] Directional light from the system's star class; flat shading reads clearly against the starfield
- [ ] Style switch in the viewer; falls back when a model lacks faces

### Phase 4: Textured and lit
- [ ] UVs and texture maps (albedo, normal, emission for lit windows and engines) for the ships
- [ ] `textured` style with light, plus planet shading from the system's star
- [ ] Engine glow and shield effects reworked in the style (overlaps #199)

### Later
- [ ] Flight and docking view (#187) uses these models and styles
- [ ] iOS export (#184): check the Mobile renderer and gdext on device

## Acceptance Criteria
- [ ] Adding a ship means adding a model file and, optionally, its ship id in `ships.json`; no scene or Rust code changes
- [ ] Switching style changes only the look; the sim and `elite-ffi` are unchanged and all Rust tests pass
- [ ] The wireframe journey scene looks the same as today after Phase 1
- [ ] Every ship in `ships.json` has a model, and the shipyard shows it
- [ ] A model missing faces or textures still renders in a lower style

## Out of scope
- New gameplay, balance changes, sound, multiplayer
- Replacing the Rust core or CLI, or porting sim logic to GDScript
- Playable flight and docking (#187), tracked separately

## Verified against current docs
- **C# on iOS** (not used here): supported experimentally since Godot 4.2, with limited simulator support and macOS-only export; web export is unsupported. Not relevant because the sim stays in Rust.
- **gdext on iOS**: reported as experimental with thin documentation (README mirrors, not the official book). Unverified; tracked as #184.
- **Headless testing**: GDScript test frameworks (GUT, gdUnit4) run headless in CI, but are not needed since logic stays in Rust; model validation is a Rust test.

## Decisions
- **Authoring**: hand-written JSON for wireframes; glTF for the textured models later, resolved by the same ids
- **Hidden-line look**: pure black fill; revisit if it looks harsh against the glow
- **Mobile renderer**: switch in Phase 1

## Dependencies
- Feature 15 (GUI shell, journey scene), Feature 22 (star class for lighting), #199 (combat effects polish), #187 (flight), #184 (iOS)
