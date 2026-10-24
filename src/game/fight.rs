use crate::game::combat::{Action, Combat, CombatError, Combatant, Outcome};
use colored::*;
use std::io::{self, BufRead, Write};

/// Play a fight to its end, reading actions from `input` and writing the fight to `out`.
/// Running out of input counts as surrender.
pub fn run_fight(
    combat: &mut Combat,
    input: &mut impl BufRead,
    out: &mut impl Write,
    roll: &mut impl FnMut() -> f64,
) -> io::Result<Outcome> {
    writeln!(out)?;
    writeln!(
        out,
        "{}",
        format!("Combat: {} vs {}", combat.player.name, combat.enemy.name)
            .red()
            .bold()
    )?;

    while combat.outcome() == Outcome::Ongoing {
        show_status(combat, out)?;

        let action = match read_action(input, out)? {
            Some(action) => action,
            None => {
                writeln!(out, "No input: you surrender.")?;
                Action::Surrender
            }
        };

        let logged = combat.log.len();
        match combat.resolve_round(action, roll) {
            Ok(_) => {
                for line in &combat.log[logged..] {
                    writeln!(out, "  {}", line)?;
                }
            }
            Err(CombatError::NoWeapons) => {
                writeln!(out, "{}", "You have no weapons installed.".red())?;
            }
            Err(error) => {
                writeln!(out, "{}", error.to_string().red())?;
                break;
            }
        }
    }

    writeln!(out)?;
    writeln!(out, "{}", ending(combat.outcome()))?;
    Ok(combat.outcome())
}

fn show_status(combat: &Combat, out: &mut impl Write) -> io::Result<()> {
    writeln!(out)?;
    writeln!(out, "{}", status_line(&combat.player))?;
    writeln!(out, "{}", status_line(&combat.enemy))?;
    Ok(())
}

fn status_line(fighter: &Combatant) -> String {
    format!(
        "{:<14} Hull {:>3}/{:<3}  Shield {:>3}/{:<3}",
        fighter.name, fighter.hull, fighter.max_hull, fighter.shield, fighter.max_shield
    )
}

/// Ask until the player gives a valid action; `None` when input has ended
fn read_action(input: &mut impl BufRead, out: &mut impl Write) -> io::Result<Option<Action>> {
    loop {
        write!(out, "[a]ttack, [f]lee or [s]urrender? ")?;
        out.flush()?;

        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }

        match line.trim().to_lowercase().as_str() {
            "a" | "attack" => return Ok(Some(Action::Attack)),
            "f" | "flee" => return Ok(Some(Action::Flee)),
            "s" | "surrender" => return Ok(Some(Action::Surrender)),
            other => writeln!(out, "Unknown action: '{}'", other)?,
        }
    }
}

fn ending(outcome: Outcome) -> String {
    match outcome {
        Outcome::Victory => "Victory! The enemy ship is destroyed.".green().to_string(),
        Outcome::Defeat => "Defeat. Your ship is disabled.".red().to_string(),
        Outcome::Escaped => "You escaped the fight.".yellow().to_string(),
        Outcome::Surrendered => "You surrendered.".yellow().to_string(),
        Outcome::Ongoing => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::combat::Weapon;
    use std::io::Cursor;

    fn fighter(name: &str, hull: u32, shield: u32, weapons: &[(u32, f64)]) -> Combatant {
        Combatant {
            name: name.to_string(),
            hull,
            max_hull: hull,
            shield,
            max_shield: shield,
            recharge: 0,
            weapons: weapons
                .iter()
                .map(|(damage, accuracy)| Weapon {
                    damage: *damage,
                    accuracy: *accuracy,
                })
                .collect(),
        }
    }

    fn play(combat: &mut Combat, typed: &str, dice: &[f64]) -> (Outcome, String) {
        colored::control::set_override(false);
        let mut remaining = dice.iter();
        let mut roll = move || *remaining.next().expect("more rolls than expected");
        let mut out = Vec::new();
        let outcome = run_fight(
            combat,
            &mut Cursor::new(typed.to_string()),
            &mut out,
            &mut roll,
        )
        .unwrap();
        (outcome, String::from_utf8(out).unwrap())
    }

    fn duel() -> Combat {
        Combat::new(
            fighter("Your ship", 100, 20, &[(40, 1.0)]),
            fighter("Raider", 60, 0, &[(10, 1.0)]),
        )
    }

    #[test]
    fn shows_both_sides_hull_and_shield_each_round() {
        let mut combat = duel();

        let (_, shown) = play(&mut combat, "a\na\n", &[0.0, 0.0, 0.0]);

        assert!(shown.contains("Combat: Your ship vs Raider"));
        assert!(shown.contains("Your ship"));
        assert!(shown.contains("Hull 100/100"));
        assert!(shown.contains("Shield  20/20"));
        assert!(shown.contains("Hull  60/60"));
        assert!(shown.contains("Hull  20/60"));
    }

    #[test]
    fn prints_what_happened_each_round() {
        let mut combat = duel();

        let (_, shown) = play(&mut combat, "a\na\n", &[0.0, 0.0, 0.0]);

        assert!(shown.contains("Your ship hits Raider for 40 damage."));
        assert!(shown.contains("Raider hits Your ship for 10 damage."));
        assert!(shown.contains("The Raider is destroyed!"));
    }

    #[test]
    fn a_won_fight_ends_with_victory() {
        let mut combat = duel();

        let (outcome, shown) = play(&mut combat, "a\na\n", &[0.0, 0.0, 0.0]);

        assert_eq!(outcome, Outcome::Victory);
        assert!(shown.contains("Victory!"));
    }

    #[test]
    fn single_letters_and_full_words_are_accepted_in_any_case() {
        let mut combat = duel();

        let (outcome, _) = play(&mut combat, "FLEE\n", &[0.1]);

        assert_eq!(outcome, Outcome::Escaped);
    }

    #[test]
    fn surrender_ends_the_fight() {
        let mut combat = duel();

        let (outcome, shown) = play(&mut combat, "s\n", &[]);

        assert_eq!(outcome, Outcome::Surrendered);
        assert!(shown.contains("You surrendered."));
    }

    #[test]
    fn an_unknown_action_asks_again_without_taking_a_turn() {
        let mut combat = duel();

        let (outcome, shown) = play(&mut combat, "dance\ns\n", &[]);

        assert!(shown.contains("Unknown action: 'dance'"));
        assert_eq!(outcome, Outcome::Surrendered);
        assert_eq!(combat.player.hull, 100);
    }

    #[test]
    fn attacking_without_weapons_explains_and_asks_again() {
        let mut combat = Combat::new(
            fighter("Your ship", 100, 0, &[]),
            fighter("Raider", 60, 0, &[(10, 1.0)]),
        );

        let (outcome, shown) = play(&mut combat, "a\ns\n", &[]);

        assert!(shown.contains("You have no weapons installed."));
        assert_eq!(outcome, Outcome::Surrendered);
        assert_eq!(combat.player.hull, 100);
    }

    #[test]
    fn running_out_of_input_counts_as_surrender() {
        let mut combat = duel();

        let (outcome, shown) = play(&mut combat, "", &[]);

        assert_eq!(outcome, Outcome::Surrendered);
        assert!(shown.contains("No input"));
    }

    #[test]
    fn a_lost_fight_ends_with_defeat() {
        let mut combat = Combat::new(
            fighter("Your ship", 5, 0, &[(1, 0.0)]),
            fighter("Raider", 60, 0, &[(10, 1.0)]),
        );

        let (outcome, shown) = play(&mut combat, "a\n", &[0.9, 0.0]);

        assert_eq!(outcome, Outcome::Defeat);
        assert!(shown.contains("Defeat."));
    }

    #[test]
    fn a_failed_escape_continues_the_fight() {
        let mut combat = duel();

        let (outcome, shown) = play(&mut combat, "f\ns\n", &[0.9, 0.0]);

        assert!(shown.contains("You fail to get away."));
        assert_eq!(outcome, Outcome::Surrendered);
        assert_eq!(combat.player.shield, 10);
    }
}
