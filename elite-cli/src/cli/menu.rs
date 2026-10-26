use crate::utils::display::{header, Icon};
use colored::*;

struct Entry {
    name: &'static str,
    aliases: &'static [&'static str],
    lines: &'static [(&'static str, &'static str)],
}

struct Section {
    icon: Icon,
    title: &'static str,
    entries: &'static [Entry],
}

const SECTIONS: &[Section] = &[
    Section {
        icon: Icon::Help,
        title: "Available Commands",
        entries: &[
            Entry {
                name: "help",
                aliases: &["h", "?"],
                lines: &[("help", "Show this help menu")],
            },
            Entry {
                name: "status",
                aliases: &["s"],
                lines: &[("status", "Display ship and commander status")],
            },
            Entry {
                name: "systems",
                aliases: &["sys"],
                lines: &[("systems", "List all available star systems")],
            },
            Entry {
                name: "info",
                aliases: &[],
                lines: &[("info <system>", "Show information about a system")],
            },
            Entry {
                name: "jump",
                aliases: &["j"],
                lines: &[("jump <system>", "Travel to a star system (with events!)")],
            },
            Entry {
                name: "route",
                aliases: &[],
                lines: &[("route <system>", "Plan the cheapest-fuel route to a star")],
            },
        ],
    },
    Section {
        icon: Icon::Market,
        title: "Trading Commands",
        entries: &[
            Entry {
                name: "market",
                aliases: &["m"],
                lines: &[("market", "Show current system's market prices")],
            },
            Entry {
                name: "buy",
                aliases: &[],
                lines: &[("buy <item> <quantity>", "Buy commodities")],
            },
            Entry {
                name: "sell",
                aliases: &[],
                lines: &[
                    ("sell <item> <quantity|all>", "Sell commodities"),
                    ("sell all", "Sell your whole hold"),
                ],
            },
            Entry {
                name: "cargo",
                aliases: &["c"],
                lines: &[("cargo", "Show cargo hold contents")],
            },
            Entry {
                name: "log",
                aliases: &[],
                lines: &[("log [count]", "Show trading history and statistics")],
            },
        ],
    },
    Section {
        icon: Icon::Ship,
        title: "Ship Upgrade Commands",
        entries: &[
            Entry {
                name: "upgrade",
                aliases: &["u"],
                lines: &[
                    ("upgrade", "Show available ship upgrades"),
                    ("upgrade <ship>", "Purchase ship upgrade"),
                ],
            },
            Entry {
                name: "ships",
                aliases: &[],
                lines: &[("ships", "List all available ships")],
            },
        ],
    },
    Section {
        icon: Icon::Equipment,
        title: "Equipment Commands",
        entries: &[Entry {
            name: "equipment",
            aliases: &["e"],
            lines: &[
                ("equipment", "Show equipment for sale"),
                ("equipment buy <item>", "Buy and install equipment"),
                (
                    "equipment sell <item>",
                    "Sell installed equipment (half price)",
                ),
                (
                    "equipment installed",
                    "Show installed equipment, slots and power",
                ),
            ],
        }],
    },
    Section {
        icon: Icon::Jobs,
        title: "Job Commands",
        entries: &[
            Entry {
                name: "jobs",
                aliases: &[],
                lines: &[("jobs", "Show the jobs on offer here")],
            },
            Entry {
                name: "accept",
                aliases: &[],
                lines: &[("accept <number>", "Take a job from the board")],
            },
            Entry {
                name: "missions",
                aliases: &[],
                lines: &[("missions", "List your active jobs")],
            },
            Entry {
                name: "abandon",
                aliases: &[],
                lines: &[("abandon <number>", "Drop an active job (-3 reputation)")],
            },
        ],
    },
    Section {
        icon: Icon::Repair,
        title: "Repair and Refuel Commands",
        entries: &[
            Entry {
                name: "repair",
                aliases: &["r"],
                lines: &[(
                    "repair [points]",
                    "Repair hull damage (2 credits per point)",
                )],
            },
            Entry {
                name: "refuel",
                aliases: &["f"],
                lines: &[("refuel [units]", "Refuel (2 credits per light year)")],
            },
        ],
    },
    Section {
        icon: Icon::Save,
        title: "Save/Load Commands",
        entries: &[
            Entry {
                name: "save",
                aliases: &[],
                lines: &[("save <slot>", "Save game to slot")],
            },
            Entry {
                name: "load",
                aliases: &[],
                lines: &[("load <slot>", "Load game from slot")],
            },
            Entry {
                name: "saves",
                aliases: &[],
                lines: &[("saves", "List all save files")],
            },
            Entry {
                name: "delete",
                aliases: &[],
                lines: &[("delete <slot>", "Delete save file")],
            },
        ],
    },
];

const QUIT: Entry = Entry {
    name: "quit",
    aliases: &["q", "exit"],
    lines: &[("quit", "Exit the game")],
};

fn entries() -> impl Iterator<Item = &'static Entry> {
    SECTIONS
        .iter()
        .flat_map(|section| section.entries.iter())
        .chain(std::iter::once(&QUIT))
}

/// The command a typed word stands for: a menu number, an alias or the command itself
pub fn resolve(word: &str) -> String {
    if let Ok(number) = word.parse::<usize>() {
        if let Some(entry) = number.checked_sub(1).and_then(|i| entries().nth(i)) {
            return entry.name.to_string();
        }
    }
    entries()
        .find(|entry| entry.aliases.contains(&word))
        .map_or_else(|| word.to_string(), |entry| entry.name.to_string())
}

fn print_entry(number: usize, entry: &Entry) {
    for (index, (usage, summary)) in entry.lines.iter().enumerate() {
        let label = if index == 0 {
            format!("{:>2}.", number)
        } else {
            "   ".to_string()
        };
        println!("{} {} - {}", label.dimmed(), usage.yellow(), summary);
    }
}

/// `help`: every command, numbered so a number can be typed instead of its name
pub fn show_help() {
    let mut number = 0;
    for section in SECTIONS {
        header(section.icon, section.title);
        for entry in section.entries {
            number += 1;
            print_entry(number, entry);
        }
    }
    println!();
    print_entry(number + 1, &QUIT);
    println!();
    println!("Type a number or a command name. Shortcuts: s status, m market, c cargo, j jump, e equipment, u upgrade, r repair, f refuel, q quit.");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_resolve_to_commands_in_help_order() {
        assert_eq!(resolve("1"), "help");
        assert_eq!(resolve("2"), "status");
        assert_eq!(resolve("7"), "market");
        assert_eq!(resolve("25"), "quit");
    }

    #[test]
    fn numbers_outside_the_menu_are_left_alone() {
        assert_eq!(resolve("0"), "0");
        assert_eq!(resolve("26"), "26");
    }

    #[test]
    fn aliases_resolve_to_commands() {
        for (alias, name) in [
            ("s", "status"),
            ("m", "market"),
            ("c", "cargo"),
            ("j", "jump"),
            ("e", "equipment"),
            ("u", "upgrade"),
            ("r", "repair"),
            ("f", "refuel"),
            ("h", "help"),
            ("?", "help"),
            ("q", "quit"),
            ("exit", "quit"),
        ] {
            assert_eq!(resolve(alias), name);
        }
    }

    #[test]
    fn full_names_and_unknown_words_pass_through() {
        assert_eq!(resolve("status"), "status");
        assert_eq!(resolve("dance"), "dance");
    }

    #[test]
    fn no_alias_is_used_twice_or_shadows_a_command() {
        let mut seen = std::collections::HashSet::new();
        for entry in entries() {
            assert!(seen.insert(entry.name), "{}", entry.name);
        }
        for entry in entries() {
            for alias in entry.aliases {
                assert!(seen.insert(alias), "{}", alias);
            }
        }
    }

    #[test]
    fn help_prints() {
        show_help();
    }
}
