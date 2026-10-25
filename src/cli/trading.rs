use crate::game::commander::Standing;
use crate::game::trade_log::TradeKind;
use crate::game::trading::{CargoView, MarketView, Pushed, TradeReceipt};
use crate::game::GameState;
use crate::utils::display::{show_error, show_info, show_success};
use colored::*;

/// `buy <item> [quantity]`
pub fn buy(game: &mut GameState, args: &[String]) -> bool {
    let Some((commodity, quantity)) = parse_trade(game, args, "buy") else {
        return false;
    };
    report(game.buy(commodity, quantity))
}

/// `sell <item> [quantity]`
pub fn sell(game: &mut GameState, args: &[String]) -> bool {
    let Some((commodity, quantity)) = parse_trade(game, args, "sell") else {
        return false;
    };
    report(game.sell(commodity, quantity))
}

pub fn market(game: &mut GameState) {
    match game.market_overview() {
        Some(view) => print_market(&view),
        None => show_error("System not found"),
    }
}

pub fn cargo(game: &GameState) {
    print_cargo(&game.cargo_view());
}

fn parse_trade(
    game: &GameState,
    args: &[String],
    verb: &str,
) -> Option<(crate::data::commodities::CommodityType, u32)> {
    if args.is_empty() {
        show_error(&format!("Usage: {} <item> [quantity]", verb));
        return None;
    }
    let name = &args[0];
    let Some(commodity) = game.trading_handler().parse_commodity(name) else {
        show_error(&format!("Unknown commodity: {}", name));
        return None;
    };
    let quantity = args.get(1).map(|s| s.as_str()).unwrap_or("1");
    match quantity.parse::<u32>() {
        Ok(quantity) if quantity > 0 => Some((commodity, quantity)),
        _ => {
            show_error("Quantity must be a positive number");
            None
        }
    }
}

fn report(result: Result<TradeReceipt, crate::game::trading::TradeError>) -> bool {
    match result {
        Ok(receipt) => {
            print_receipt(&receipt);
            true
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

fn print_receipt(receipt: &TradeReceipt) {
    let verb = match receipt.kind {
        TradeKind::Buy => "Purchased",
        TradeKind::Sell => "Sold",
    };
    show_success(&format!(
        "{} {} tons of {} for {}cr",
        verb,
        receipt.quantity,
        receipt.commodity.name(),
        receipt.total
    ));
    if let Some(change) = receipt.illegal_penalty {
        println!(
            "{}",
            format!(
                "Trading {} is illegal. Reputation {:+}.",
                receipt.commodity.name(),
                change.applied
            )
            .yellow()
        );
        if let Some(standing) = change.new_standing {
            show_info(&format!("Your standing is now {}.", standing.name()));
        }
    }
}

fn markers(pushed: Pushed) -> String {
    pushed.markers().join(" ")
}

fn print_market(view: &MarketView) {
    println!("\n{} Market", view.system_name.yellow());
    println!("{}", "=".repeat(view.system_name.len() + 7));
    if view.standing != Standing::Neutral {
        println!(
            "Prices adjusted for your standing: {} ({:+}% on trades)",
            view.standing.name(),
            view.standing.price_favour_percent()
        );
    }

    for row in &view.rows {
        let name = format!("{}:", row.commodity.name());
        let buy = format!("Buy: {}cr", row.buy_price);
        let sell = format!("Sell: {}cr", row.sell_price);
        let available = format!("Available: {} tons", row.available);
        let versus_base = format!("{:<14}", format!("{:+}% vs base", row.percent_vs_base));
        let versus_base = if row.percent_vs_base <= -10 {
            versus_base.green()
        } else if row.percent_vs_base >= 10 {
            versus_base.red()
        } else {
            versus_base.normal()
        };
        let holding = match (row.held, row.paid) {
            (0, _) => String::new(),
            (held, Some(paid)) => format!("Hold: {}t @ {}cr", held, paid),
            (held, None) => format!("Hold: {}t", held),
        };
        println!(
            "{:<12} {:<12} {:<12} {:<20} {} {} {}",
            name,
            buy,
            sell,
            available,
            versus_base,
            holding.cyan(),
            markers(row.pushed).yellow()
        );
    }
    println!();
}

fn print_cargo(view: &CargoView) {
    println!("\nCargo Hold ({}/{})", view.weight, view.capacity);
    println!("{}", "=".repeat(20));

    if view.reserved > 0 {
        println!("Contract cargo: {} tons (not for sale)", view.reserved);
    }
    if view.items.is_empty() {
        println!("No goods in the hold");
    } else {
        for line in &view.items {
            let paid = line
                .paid
                .map(|paid| format!("  Paid: {}cr/ton", paid))
                .unwrap_or_default();
            println!(
                "{:<12} {} tons  Value: {}cr{}",
                format!("{}:", line.commodity.name()),
                line.quantity,
                line.estimated_value,
                paid
            );
        }
        println!("{}", "-".repeat(20));
        println!("Total Value: {}cr", view.total_value);
    }
    println!();
}
