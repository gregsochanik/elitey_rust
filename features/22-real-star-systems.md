# Feature 22: Real Star Systems

## Overview
Replace the invented system names with real stars (Sol, Sirius, Tau Ceti, Barnard's Star), placed where they really are.

## Status
Done (v1).

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
- Each description is two to three sentences built from the star's spectral class, economy and government, picked per star from a fixed seed; the best-known stars (Sol, Sirius, Tau Ceti and others) have hand-written ones
- Each system has a `star_class` from HYG (`G2V`); the info screens show a plain name and a one-line note (`Star: Yellow dwarf (G2V)`) so players learn what the letters mean. Saves that predate it show no star line
- Every system has at least 2 others within 7 ly (the starting fuel range) and the galaxy is connected, so nobody is stranded; the script drops stars that would break this
- Credit HYG and the licence in the README and on the GUI start screen; `galaxy1.json` stays CC BY-SA 4.0

## Decisions
- **Old saves**: each save carries the galaxy it was made in, so old saves keep playing in the old galaxy and nothing needs refusing or migrating; new games and new saves use the real stars
- **Distances**: real positions are scaled so the nearest neighbours stay 2 to 4 ly apart, keeping fuel and jump economics unchanged
- **Names**: multi-word and punctuated names work in `jump` and `info`; tests move from `LAVE`/`DISO` to Sol and its neighbours

## Tests
- Generated file has 256 unique systems, all within the 0 to 255 grid, none overlapping
- Sol, Sirius and Barnard's Star are present with the right relative positions
- Regenerating from the same HYG file and seed gives an identical file
- Existing economy, jobs and journey tests pass with the new galaxy

## Not in v1
- Real planets, 3D map, other galaxies
