use crate::game::trade_log::TradeKind;
use crate::game::views::{StatusView, SystemInfoView, TradeLogView};
use crate::game::GameState;
use crate::utils::display::show_error;
use colored::*;

/// `status`
pub fn status(game: &GameState) {
    print_status(&game.status_view());
}

/// `log [count]`
pub fn log(game: &GameState, args: &[String]) {
    let count = match args.first() {
        None => 10,
        Some(arg) => match arg.parse::<usize>() {
            Ok(n) if n > 0 => n,
            _ => {
                show_error("Usage: log [count]");
                return;
            }
        },
    };
    print_trade_log(&game.trade_log_view(count));
}

/// `systems`
pub fn systems(game: &GameState) {
    let Some(reachable) = game.reachable_destinations() else {
        show_error("Unable to determine current location");
        return;
    };

    println!();
    println!("{}", "Available Star Systems".cyan().bold());
    println!("{}", "=====================".cyan());
    println!("Current fuel: {} light years", game.ship.fuel_current);
    println!();

    if reachable.is_empty() {
        println!("{}", "No systems reachable with current fuel.".yellow());
        println!("{}", "Use 'refuel' to top up the tank.".yellow());
    } else {
        for destination in reachable {
            println!(
                "{} - {:.1} light years, {} fuel",
                destination.name.yellow(),
                destination.distance,
                destination.fuel
            );
        }
    }
    println!();
}

/// `info <system>`
pub fn system_info(game: &GameState, args: &[String]) -> bool {
    let Some(name) = args.first() else {
        show_error("Usage: info <system_name>");
        return false;
    };
    match game.system_info(name) {
        Some(view) => {
            print_system_info(&view);
            true
        }
        None => {
            show_error(&format!("System '{}' not found", name));
            false
        }
    }
}

fn print_status(view: &StatusView) {
    println!();
    println!("{}", "Commander Status".cyan().bold());
    println!("{}", "================".cyan());
    println!("Commander: {}", view.commander_name.yellow());
    println!("Credits: {} cr", view.credits.to_string().green());
    println!("Location: {}", view.location.blue());
    println!("Reputation: {} ({})", view.reputation, view.standing.name());
    println!();
    println!("{}", "Ship Status".cyan().bold());
    println!("{}", "===========".cyan());
    println!("Ship: {}", view.ship_name.yellow());
    println!("Cargo: {}/{} tons", view.cargo_used, view.cargo_capacity);
    println!(
        "Fuel: {}/{} light years",
        view.fuel_current, view.fuel_capacity
    );
    println!("Hull: {}/{}", view.hull, view.max_hull);

    if view.damaged {
        println!("{}", "⚠️  Hull damage detected!".red());
    }
    if view.fuel_current < view.fuel_capacity / 3 {
        println!("{}", "⚠️  Low fuel warning!".yellow());
    }
    if view.cargo_used > view.cargo_capacity * 3 / 4 {
        println!("{}", "⚠️  Cargo hold nearly full!".yellow());
    }
    println!();
}

fn print_system_info(view: &SystemInfoView) {
    println!();
    println!("{}", format!("System: {}", view.name).cyan().bold());
    println!("{}", "=".repeat(30));
    println!("Economy: {:?}", view.economy);
    println!("Government: {:?}", view.government);
    println!("Tech Level: {}", view.tech_level);
    println!("Population: {}", view.population);
    println!("Description: {}", view.description);
    if let Some(jump) = &view.from_here {
        println!("Distance: {:.1} light years", jump.distance);
        println!("Fuel required: {} light years", jump.fuel);
    }
    println!();
}

fn print_trade_log(view: &TradeLogView) {
    println!("\n{}", "Trading Log".cyan().bold());
    println!("{}", "===========".cyan());

    if view.recent.is_empty() {
        println!("No trades yet.\n");
        return;
    }

    println!("Recent Transactions:");
    for t in &view.recent {
        let name = t.commodity.name();
        match (t.kind, t.profit) {
            (TradeKind::Sell, Some(profit)) => {
                let profit_str = format!("{:+}cr profit", profit);
                let coloured = if profit >= 0 {
                    profit_str.green()
                } else {
                    profit_str.red()
                };
                println!(
                    "- Sold {} {} at {} for {}cr ({})",
                    t.quantity,
                    name,
                    t.location,
                    t.total(),
                    coloured
                );
            }
            _ => println!(
                "- Bought {} {} at {} for {}cr",
                t.quantity,
                name,
                t.location,
                t.total()
            ),
        }
    }

    println!("\nStatistics:");
    println!("- Total Trades: {}", view.stats.total_trades);
    println!("- Net Profit: {}cr", view.stats.net_profit);
    if let Some(best) = &view.best {
        println!(
            "- Best Trade: {:+}cr ({} {} at {})",
            best.profit.unwrap_or(0),
            best.quantity,
            best.commodity.name(),
            best.location
        );
    }
    if let Some(rate) = view.stats.success_rate {
        println!("- Success Rate: {:.0}%", rate * 100.0);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_status_view_reports_the_ship_and_commander() {
        let mut game = GameState::new("Test".to_string());
        game.ship.take_damage(5);
        game.ship.fuel_current = 1;

        let view = game.status_view();

        assert_eq!(view.commander_name, "Test");
        assert_eq!(view.location, "LAVE");
        assert_eq!((view.hull, view.max_hull), (95, 100));
        assert!(view.damaged);
        assert_eq!(view.fuel_current, 1);
        print_status(&view);
    }

    #[test]
    fn the_system_info_view_includes_the_jump_from_here() {
        let game = GameState::new("Test".to_string());

        let diso = game.system_info("DISO").unwrap();
        assert_eq!(diso.name, "DISO");
        assert_eq!(diso.from_here.as_ref().unwrap().fuel, 3);
        assert!(game.system_info("NOWHERE").is_none());
        print_system_info(&diso);
    }

    #[test]
    fn the_trade_log_view_limits_recent_trades() {
        let mut game = GameState::new("Test".to_string());
        game.commander.credits = 1_000;
        for _ in 0..3 {
            game.buy(crate::data::commodities::CommodityType::Food, 1)
                .unwrap();
        }

        let view = game.trade_log_view(2);

        assert_eq!(view.recent.len(), 2);
        assert_eq!(view.stats.total_trades, 3);
        print_trade_log(&view);
    }
}
