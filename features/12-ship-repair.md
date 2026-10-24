# Feature 12: Ship Repair

## Overview
Combat leaves the hull damaged. `repair` restores it for credits so the player can keep fighting.

## Version 1 Scope
- `repair` restores the hull to full, or as far as the player can afford
- `repair <points>` restores that many points, capped at the damage taken
- Cost is 2 credits per hull point; available in any system (there are no stations yet)
- Hull never exceeds the ship's maximum (upgraded ships have more than 100)
- No credits are spent when there is nothing to repair, or when the player cannot afford a single point

## Not in v1
- Station-only repair, price variation by system or government
- Equipment damage

## Balance
The starting ship loses about 28 hull per fight, so repair costs about 56 credits per fight. A pirate victory pays 200–400 credits.

## Acceptance Criteria
- [x] `repair` on a damaged ship restores the hull and charges 2 credits per point
- [x] Not enough credits repairs as many points as affordable
- [x] `repair <points>` repairs only that many points
- [x] A full-hull ship is refused and nothing is charged
- [x] A ship with no credits is refused and the hull is unchanged
- [x] Repair never exceeds the ship's maximum hull
- [x] Invalid arguments show usage

## Testing (write first)
- [x] Unit tests for each criterion above
- [x] Flow test: a ship damaged in combat is repaired and credits drop by the expected amount

## Dependencies
- Feature 08: Combat ✅
