# Feature 14: Refuelling

## Overview
Jumps burn fuel and nothing restores it, so a ship can strand itself. `refuel` tops up the tank for credits.

## Version 1 Scope
- `refuel` fills the tank, or as far as the player can afford
- `refuel <units>` adds that many light years, capped at the tank's capacity
- Cost is 2 credits per light year; available in any system (there are no stations yet)
- No credits are spent when the tank is full or the player cannot afford one unit

## Rescue
A player with an empty hold and too little fuel to reach any system, who cannot afford enough fuel (`fuel + credits / 2` is below the nearest system's fuel cost), is stranded. `refuel` then rescues them once: the tank is topped up to reach the nearest system for free, and reputation falls by 2.

## Not in v1
- Station-only refuelling, prices that vary by system or government

## Acceptance Criteria
- [x] `refuel` on a part-empty tank fills it and charges 2 credits per light year
- [x] Not enough credits refuels as many units as affordable
- [x] `refuel <units>` refuels only that many, never past capacity
- [x] A full tank is refused and nothing is charged
- [x] A ship with no credits is refused and the tank is unchanged
- [x] Capacity follows the ship (upgraded ships hold more)
- [x] Invalid arguments show usage
- [x] A stranded ship is rescued for 2 reputation and can jump again; no rescue while there is cargo to sell or enough credits to move

## Dependencies
- Feature 02: Star Systems & Travel ✅
