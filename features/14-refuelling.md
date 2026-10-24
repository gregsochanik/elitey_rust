# Feature 14: Refuelling

## Overview
Jumps burn fuel and nothing restores it, so a ship can strand itself. `refuel` tops up the tank for credits.

## Version 1 Scope
- `refuel` fills the tank, or as far as the player can afford
- `refuel <units>` adds that many light years, capped at the tank's capacity
- Cost is 2 credits per light year; available in any system (there are no stations yet)
- No credits are spent when the tank is full or the player cannot afford one unit

## Not in v1
- Station-only refuelling, prices that vary by system or government
- Rescue when stranded with no credits (see the balance issue)

## Acceptance Criteria
- [x] `refuel` on a part-empty tank fills it and charges 2 credits per light year
- [x] Not enough credits refuels as many units as affordable
- [x] `refuel <units>` refuels only that many, never past capacity
- [x] A full tank is refused and nothing is charged
- [x] A ship with no credits is refused and the tank is unchanged
- [x] Capacity follows the ship (upgraded ships hold more)
- [x] Invalid arguments show usage

## Dependencies
- Feature 02: Star Systems & Travel ✅
