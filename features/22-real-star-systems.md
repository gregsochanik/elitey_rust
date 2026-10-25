# Feature 22: Real Star Systems

## Overview
Replace the invented system names with real stars (Sol, Sirius, Tau Ceti, Barnard's Star), placed where they really are.

## Status
Proposed. Open decisions are marked **Decide**.

## Source
HYG star database v4.4 (<https://codeberg.org/astronexus/hyg>, `data/hyg/CURRENT/hyg_v44.csv.gz`), CC BY-SA 4.0. Columns used: `proper`, `bf` (Bayer/Flamsteed), `gl` (Gliese), `dist` (parsecs), `x`/`y`/`z` (parsecs, Sol at origin), `spect`, `comp_primary`.

Of 119,614 stars, 549 have proper names; 149 of those lie within 100 ly of Sol.

## Version 1 Scope
- `galaxy1.json` is generated once by a script (`tools/`), not read from HYG at runtime; the generated file is committed
- 256 systems, the nearest stars to Sol, so the map keeps its current size
- Name: `proper`, else Bayer + constellation (`Alpha Centauri`), else Gliese (`GJ 1061`)
- One system per star system: companions share their primary (Toliman and Rigil Kentaurus are one system, Alpha Centauri)
- Position: HYG `x`/`y` in light years, scaled to fit the existing 0 to 255 grid; `z` is dropped
- Sol is the starting system, replacing LAVE
- Economy, government, tech level, population and description are generated per star from a fixed seed, so the file is reproducible; real stars carry none of these
- Spectral class is shown on the system info screen
- Credit HYG and the licence in the README and on the GUI start screen; `galaxy1.json` stays CC BY-SA 4.0

## Decide
- **Old saves**: they name systems that no longer exist. Options: refuse with a clear message (recommended: pre-1.0 game, few saves), or map old names to the nearest new system
- **Distances**: scaling real light years onto the grid makes jump costs and fuel range depend on the scale factor. Keep the current fuel economics by scaling so the nearest neighbours stay 2 to 4 ly apart (recommended), or use true distances and rebalance
- **Names with spaces/punctuation**: `Barnard's Star` breaks the unquoted `jump <system>` CLI command and some tests (`LAVE`, `DISO`); allow multi-word names in `jump`/`info` (already joins args?) and update tests

## Tests
- Generated file has 256 unique systems, all within the 0 to 255 grid, none overlapping
- Sol, Sirius and Barnard's Star are present with the right relative positions
- Regenerating from the same HYG file and seed gives an identical file
- Existing economy, jobs and journey tests pass with the new galaxy

## Not in v1
- Real planets, 3D map, other galaxies
