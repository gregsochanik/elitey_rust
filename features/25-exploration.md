# Feature 25: Exploration

## Overview
The game keeps a log of the stars the commander has visited and rates how much of the galaxy they have seen.

## Status
Done (v1).

## Version 1 Scope
- The commander keeps a set of visited systems, saved with the game. Sol counts from the start; a system counts when the commander arrives there (including a diversion)
- Saves from before the log count the system the commander is in
- Exploration is visited stars over all stars in the galaxy, as a whole percent; 100 needs every star
- Ratings by percent: Local (0 to 9), Traveller (10 to 24), Explorer (25 to 49), Pathfinder (50 to 74), Cartographer (75 to 99), Master Cartographer (100)
- `status` shows `Explored: 12% — Local (31 of 256 stars)`; `info <system>` says whether it has been visited
- GUI: the galaxy chart shows the same line; unvisited stars carry a yellow ring and the details panel says whether the selected star was visited

## Not in v1
- Discoveries at frontier systems (issue #208), which build on this log
