# Feature 27: Frontier Discoveries

## Overview
The outer stars are the frontier. The first time the commander reaches one, something is found there: salvage, survey data, charts, fuel or an alien artefact. Finds are logged and the stars they chart count toward exploration (Feature 25).

## Status
In progress, in slices: #256 mark the frontier, #257 discovery data and log, #258 find on arrival, #259 CLI, #260 GUI.

## Version 1 Scope

### Frontier
- The farthest 12% of stars from Sol (31 of 256) are frontier systems. It is computed from positions, so the galaxy file does not change and saves carrying an older galaxy work too (they measure from the middle of the map when there is no Sol)
- `info` and the GUI chart and details mark them (a blue ring on the chart)

### Discoveries
- Each frontier star holds one find, rolled when the commander first arrives and never again
- Finds live in `data/discoveries.json`, picked by weight:

| Find | Weight | Effect |
|---|---|---|
| Derelict freighter | 4 | 80 to 200 credits and 2 to 4 tons of machinery |
| Uncharted world | 3 | 150 to 350 credits of survey data |
| Forgotten navigation beacon | 3 | charts the 3 nearest unvisited stars (they count as visited) |
| Hidden fuel cache | 2 | fills the tank |
| Alien artefact | 1 | 1 ton of Alien Artefacts |

- Alien Artefacts (base price 300) are a commodity that can only be found: markets list it only while it is in the hold, sell it at the usual prices, and never sell it
- The commander keeps a log of finds (where, which) saved with the game; `discoveries` lists it
- A find is reported in the arrival notices, and in the GUI in the arrival dialog

## Not in v1
- Discoveries that unlock equipment, and frontier-only systems beyond the current galaxy
