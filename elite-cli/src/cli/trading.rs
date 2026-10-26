use crate::utils::display::{header, show_error, show_info, show_success, Icon};
use crate::utils::table::{Table, LEFT, RIGHT};
use colored::*;
use elite_core::game::commander::Standing;
use elite_core::game::trade_log::TradeKind;
use elite_core::game::trading::{CargoView, MarketView, Pushed, TradeReceipt};
use elite_core::game::GameState;

/// `buy <item> [quantity]`
pub fn buy(game: &mut GameState, args: &[String]) -> bool {
    let Some((commodity, quantity)) = parse_trade(game, args, "buy") else {
        return false;
    };
    report(game.buy(commodity, quantity))
}

/// `sell <item> [quantity]`, `sell <item> all` or `sell all`
pub fn sell(game: &mut GameState, args: &[String]) -> bool {
    let is_all = |word: Option<&String>| word.is_some_and(|word| word.eq_ignore_ascii_case("all"));
    if is_all(args.first()) && args.len() == 1 {
        return report_all(game.sell_all(None));
    }
    if is_all(args.get(1)) {
        let name = &args[0];
        let Some(commodity) = game.trading_handler().parse_commodity(name) else {
            show_error(&format!("Unknown commodity: {}", name));
            return false;
        };
        return report_all(game.sell_all(Some(commodity)));
    }
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
) -> Option<(elite_core::data::commodities::CommodityType, u32)> {
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

fn report_all(result: Result<Vec<TradeReceipt>, elite_core::game::trading::TradeError>) -> bool {
    match result {
        Ok(receipts) => {
            for receipt in &receipts {
                print_receipt(receipt);
            }
            true
        }
        Err(error) => {
            show_error(&error.to_string());
            false
        }
    }
}

fn report(result: Result<TradeReceipt, elite_core::game::trading::TradeError>) -> bool {
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
    header(Icon::Market, &format!("{} Market", view.system_name));
    if view.standing != Standing::Neutral {
        println!(
            "Prices adjusted for your standing: {} ({:+}% on trades)",
            view.standing.name(),
            view.standing.price_favour_percent()
        );
    }

    let mut table = Table::new(&[
        ("Commodity", LEFT),
        ("Buy", RIGHT),
        ("Sell", RIGHT),
        ("Available", RIGHT),
        ("vs Base", RIGHT),
        ("Hold", LEFT),
        ("", LEFT),
    ]);
    for row in &view.rows {
        let versus_base = format!("{:+}%", row.percent_vs_base);
        let versus_base = if row.percent_vs_base <= -10 {
            versus_base.green()
        } else if row.percent_vs_base >= 10 {
            versus_base.red()
        } else {
            versus_base.normal()
        };
        let holding = match (row.held, row.paid) {
            (0, _) => String::new(),
            (held, Some(paid)) => format!("{}t @ {}cr", held, paid),
            (held, None) => format!("{}t", held),
        };
        let available = if row.restriction.is_some() {
            "not sold".red().to_string()
        } else {
            format!("{}t", row.available)
        };
        table.row([
            row.commodity.name().to_string(),
            format!("{}cr", row.buy_price),
            format!("{}cr", row.sell_price),
            available,
            versus_base.to_string(),
            holding.cyan().to_string(),
            markers(row.pushed).yellow().to_string(),
        ]);
    }
    table.print();
    for row in &view.rows {
        if let Some(reason) = &row.restriction {
            println!("{}", reason.dimmed());
        }
    }
    println!();
}

fn print_cargo(view: &CargoView) {
    header(
        Icon::Cargo,
        &format!("Cargo Hold ({}/{})", view.weight, view.capacity),
    );

    if view.reserved > 0 {
        println!("Contract cargo: {} tons (not for sale)", view.reserved);
    }
    if view.items.is_empty() {
        println!("No goods in the hold");
    } else {
        let mut table = Table::new(&[
            ("Commodity", LEFT),
            ("Tons", RIGHT),
            ("Value", RIGHT),
            ("Paid/ton", RIGHT),
        ]);
        for line in &view.items {
            table.row([
                line.commodity.name().to_string(),
                line.quantity.to_string(),
                format!("{}cr", line.estimated_value),
                line.paid
                    .map(|paid| format!("{}cr", paid))
                    .unwrap_or_default(),
            ]);
        }
        table.print();
        println!("Total Value: {}cr", view.total_value);
    }
    println!();
}
