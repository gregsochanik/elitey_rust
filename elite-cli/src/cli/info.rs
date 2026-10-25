use crate::utils::display::{
    bar, credits, economy_icon, government_icon, header, icon, show_error, system_name, Fill, Icon,
};
use crate::utils::table::{Table, LEFT, RIGHT};
use colored::*;
use elite_core::game::trade_log::TradeKind;
use elite_core::game::views::{StatusView, SystemInfoView, TradeLogView};
use elite_core::game::GameState;

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

    header(Icon::Ship, "Available Star Systems");
    println!("Current fuel: {} light years", game.ship.fuel_current);
    println!();

    if reachable.is_empty() {
        println!("{}", "No systems reachable with current fuel.".yellow());
        println!("{}", "Use 'refuel' to top up the tank.".yellow());
    } else {
        let mut table = Table::new(&[
            ("System", LEFT),
            ("Distance", RIGHT),
            ("Fuel", RIGHT),
            ("Economy", LEFT),
            ("Government", LEFT),
        ]);
        for destination in reachable {
            table.row([
                system_name(&destination.name, false).to_string(),
                format!("{:.1} ly", destination.distance),
                destination.fuel.to_string(),
                format!(
                    "{}{}",
                    economy_icon(&destination.economy),
                    destination.economy.display_name()
                ),
                format!(
                    "{}{}",
                    government_icon(&destination.government),
                    destination.government.display_name()
                ),
            ]);
        }
        table.print();
    }
    println!();
}

/// `info <system>`
pub fn system_info(game: &GameState, args: &[String]) -> bool {
    if args.is_empty() {
        show_error("Usage: info <system_name>");
        return false;
    }
    let name = args.join(" ");
    match game.system_info(&name) {
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
    header(Icon::Ship, "Commander Status");
    println!("Commander: {}", view.commander_name.yellow());
    println!(
        "{}Credits: {}",
        icon(Icon::Credits),
        credits(i64::from(view.credits))
    );
    println!(
        "{}Location: {}",
        icon(Icon::Ship),
        system_name(&view.location, true)
    );
    println!(
        "{}Reputation: {} ({})",
        icon(Icon::Rating),
        view.reputation,
        view.standing.name()
    );
    match view.combat_rating.next() {
        Some((next, needed)) => println!(
            "{}Combat rating: {} ({} kills, {} more for {})",
            icon(Icon::Combat),
            view.combat_rating.name(),
            view.kills,
            needed - view.kills,
            next.name()
        ),
        None => println!(
            "{}Combat rating: {} ({} kills)",
            icon(Icon::Combat),
            view.combat_rating.name(),
            view.kills
        ),
    }
    println!(
        "{}Explored: {}% — {} ({} of {} stars)",
        icon(Icon::Explore),
        view.exploration.percent,
        view.exploration.rating.name(),
        view.exploration.visited,
        view.exploration.total
    );
    header(Icon::Hull, "Ship Status");
    println!("{}Ship: {}", icon(Icon::Ship), view.ship_name.yellow());
    println!(
        "{}Cargo: {}/{} tons {}",
        icon(Icon::Cargo),
        view.cargo_used,
        view.cargo_capacity,
        bar(view.cargo_used, view.cargo_capacity, Fill::FullIsWarning)
    );
    println!(
        "{}Fuel: {}/{} light years {}",
        icon(Icon::Fuel),
        view.fuel_current,
        view.fuel_capacity,
        bar(view.fuel_current, view.fuel_capacity, Fill::FullIsGood)
    );
    println!(
        "{}Hull: {}/{} {}",
        icon(Icon::Hull),
        view.hull,
        view.max_hull,
        bar(view.hull, view.max_hull, Fill::FullIsGood)
    );
    println!(
        "{}Power: {}/{} {}",
        icon(Icon::Power),
        view.power_used,
        view.power_capacity,
        bar(view.power_used, view.power_capacity, Fill::FullIsWarning)
    );

    if view.damaged {
        println!(
            "{}",
            format!("{}Hull damage detected!", icon(Icon::Warning)).red()
        );
    }
    if view.fuel_current < view.fuel_capacity / 3 {
        println!(
            "{}",
            format!("{}Low fuel warning!", icon(Icon::Warning)).yellow()
        );
    }
    if view.cargo_used > view.cargo_capacity * 3 / 4 {
        println!(
            "{}",
            format!("{}Cargo hold nearly full!", icon(Icon::Warning)).yellow()
        );
    }
    println!();
}

fn print_system_info(view: &SystemInfoView) {
    header(Icon::Ship, &format!("System: {}", view.name));
    println!(
        "{}Economy: {}",
        economy_icon(&view.economy),
        view.economy.display_name()
    );
    println!(
        "{}Government: {}",
        government_icon(&view.government),
        view.government.display_name()
    );
    println!("Tech Level: {}", view.tech_level);
    println!("Population: {}", view.population);
    println!("Visited: {}", if view.visited { "yes" } else { "not yet" });
    if let Some(star) = &view.star {
        println!("Star: {} ({})", star.type_name, star.code);
        println!("      {}", star.note);
    }
    println!("Description: {}", view.description);
    if let Some(jump) = &view.from_here {
        println!("Distance: {:.1} light years", jump.distance);
        println!("Fuel required: {} light years", jump.fuel);
    }
    println!();
}

fn print_trade_log(view: &TradeLogView) {
    header(Icon::Market, "Trading Log");

    if view.recent.is_empty() {
        println!("No trades yet.\n");
        return;
    }

    println!("Recent Transactions:");
    let mut table = Table::new(&[
        ("Trade", LEFT),
        ("Qty", RIGHT),
        ("Commodity", LEFT),
        ("Location", LEFT),
        ("Total", RIGHT),
        ("Profit", RIGHT),
    ]);
    for t in &view.recent {
        let (kind, profit) = match (t.kind, t.profit) {
            (TradeKind::Sell, Some(profit)) => {
                let text = format!("{:+}cr", profit);
                (
                    "Sold",
                    if profit >= 0 {
                        text.green().to_string()
                    } else {
                        text.red().to_string()
                    },
                )
            }
            (TradeKind::Sell, None) => ("Sold", String::new()),
            _ => ("Bought", String::new()),
        };
        table.row([
            kind.to_string(),
            t.quantity.to_string(),
            t.commodity.name().to_string(),
            t.location.clone(),
            format!("{}cr", t.total()),
            profit,
        ]);
    }
    table.print();

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
        assert_eq!(view.location, "Sol");
        assert_eq!((view.hull, view.max_hull), (95, 100));
        assert!(view.damaged);
        assert_eq!(view.fuel_current, 1);
        assert_eq!((view.power_used, view.power_capacity), (9, 20));
        print_status(&view);
    }

    #[test]
    fn the_system_info_view_includes_the_jump_from_here() {
        let game = GameState::new("Test".to_string());

        let proxima = game.system_info("Proxima Centauri").unwrap();
        assert_eq!(proxima.name, "Proxima Centauri");
        assert_eq!(proxima.from_here.as_ref().unwrap().fuel, 3);
        assert!(game.system_info("NOWHERE").is_none());
        print_system_info(&proxima);
    }

    #[test]
    fn the_trade_log_view_limits_recent_trades() {
        let mut game = GameState::new("Test".to_string());
        game.commander.credits = 1_000;
        for _ in 0..3 {
            game.buy(elite_core::data::commodities::CommodityType::Food, 1)
                .unwrap();
        }

        let view = game.trade_log_view(2);

        assert_eq!(view.recent.len(), 2);
        assert_eq!(view.stats.total_trades, 3);
        print_trade_log(&view);
    }
}
