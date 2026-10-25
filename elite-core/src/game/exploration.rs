/// How much of the galaxy the commander has seen
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExplorerRating {
    Local,
    Traveller,
    Explorer,
    Pathfinder,
    Cartographer,
    MasterCartographer,
}

impl ExplorerRating {
    /// 100 only counts when every star has been visited
    pub fn from_percent(percent: u32) -> Self {
        match percent {
            0..=9 => ExplorerRating::Local,
            10..=24 => ExplorerRating::Traveller,
            25..=49 => ExplorerRating::Explorer,
            50..=74 => ExplorerRating::Pathfinder,
            75..=99 => ExplorerRating::Cartographer,
            _ => ExplorerRating::MasterCartographer,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ExplorerRating::Local => "Local",
            ExplorerRating::Traveller => "Traveller",
            ExplorerRating::Explorer => "Explorer",
            ExplorerRating::Pathfinder => "Pathfinder",
            ExplorerRating::Cartographer => "Cartographer",
            ExplorerRating::MasterCartographer => "Master Cartographer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exploration {
    pub visited: u32,
    pub total: u32,
    pub percent: u32,
    pub rating: ExplorerRating,
}

impl Exploration {
    pub fn new(visited: u32, total: u32) -> Self {
        let percent = if total == 0 || visited >= total {
            100
        } else {
            visited * 100 / total
        };
        Self {
            visited,
            total,
            percent,
            rating: ExplorerRating::from_percent(percent),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tiers_start_where_they_say() {
        for (percent, rating) in [
            (0, ExplorerRating::Local),
            (9, ExplorerRating::Local),
            (10, ExplorerRating::Traveller),
            (25, ExplorerRating::Explorer),
            (50, ExplorerRating::Pathfinder),
            (75, ExplorerRating::Cartographer),
            (99, ExplorerRating::Cartographer),
            (100, ExplorerRating::MasterCartographer),
        ] {
            assert_eq!(ExplorerRating::from_percent(percent), rating, "{percent}");
        }
    }

    #[test]
    fn a_hundred_percent_needs_every_star() {
        assert_eq!(Exploration::new(255, 256).percent, 99);
        assert_eq!(Exploration::new(256, 256).percent, 100);
        assert_eq!(Exploration::new(1, 256).percent, 0);
        assert_eq!(Exploration::new(64, 256).percent, 25);
    }
}
