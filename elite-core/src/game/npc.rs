use crate::data::enemies::Behaviour;
use crate::game::combat::{Combatant, Weapon};

/// What an enemy does in a round
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcAction {
    Attack,
    Flee,
    Surrender,
}

fn hull_fraction(fighter: &Combatant) -> f64 {
    if fighter.max_hull == 0 {
        return 0.0;
    }
    f64::from(fighter.hull) / f64::from(fighter.max_hull)
}

fn damage_per_round(weapons: &[Weapon]) -> f64 {
    weapons
        .iter()
        .map(|weapon| f64::from(weapon.damage) * weapon.accuracy)
        .sum()
}

/// How the enemy's staying power compares with the player's; above 1 favours the enemy
fn odds(npc: &Combatant, player: &Combatant) -> f64 {
    let power = |fighter: &Combatant| {
        f64::from(fighter.hull + fighter.shield) * damage_per_round(&fighter.weapons)
    };
    let player_power = power(player);
    if player_power <= 0.0 {
        return f64::INFINITY;
    }
    power(npc) / player_power
}

/// Choose the enemy's action. Surrender is tried first, then flight, otherwise it attacks.
/// `roll` returns 0.0..1.0 and is called only when a threshold is reached, once for the
/// surrender check and once for the flight check, so a fight that touches neither uses no rolls.
pub fn decide(
    behaviour: &Behaviour,
    npc: &Combatant,
    player: &Combatant,
    roll: &mut impl FnMut() -> f64,
) -> NpcAction {
    let hull = hull_fraction(npc);

    let defenceless = npc.weapons.is_empty();
    let hurt_enough_to_yield =
        behaviour.surrender_below_hull > 0.0 && hull <= behaviour.surrender_below_hull;
    if (defenceless || hurt_enough_to_yield) && roll() < behaviour.surrender_chance {
        return NpcAction::Surrender;
    }

    let nerve = 1.0 - behaviour.aggression.clamp(0.0, 1.0);
    let flee_below = behaviour.flee_below_hull * nerve;
    let flee_odds = behaviour.flee_odds * nerve;
    let hurt_enough_to_run = flee_below > 0.0 && hull <= flee_below;
    let outmatched = flee_odds > 0.0 && odds(npc, player) < flee_odds;
    if (hurt_enough_to_run || outmatched) && roll() < behaviour.flee_chance {
        return NpcAction::Flee;
    }

    NpcAction::Attack
}

#[cfg(test)]
mod tests;
