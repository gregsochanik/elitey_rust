use crate::data::star_class::StarClass;
use crate::data::systems::{EconomyType, GovernmentType};
use crate::game::commander::{CombatRating, Standing};
use crate::game::exploration::Exploration;
use crate::game::trade_log::{TradeStats, Transaction};

/// The commander and ship at a glance
#[derive(Debug, Clone, PartialEq)]
pub struct StatusView {
    pub commander_name: String,
    pub credits: u32,
    pub location: String,
    pub reputation: i32,
    pub standing: Standing,
    pub kills: u32,
    pub combat_rating: CombatRating,
    pub ship_name: String,
    pub cargo_used: u32,
    pub cargo_capacity: u32,
    pub fuel_current: u32,
    pub fuel_capacity: u32,
    pub hull: u32,
    pub max_hull: u32,
    pub power_used: u32,
    pub power_capacity: u32,
    pub damaged: bool,
    pub exploration: Exploration,
}

/// How far a system is from where the ship is
#[derive(Debug, Clone, PartialEq)]
pub struct JumpFromHere {
    pub distance: f64,
    pub fuel: u32,
}

/// What is known about a system
#[derive(Debug, Clone, PartialEq)]
pub struct SystemInfoView {
    pub name: String,
    pub economy: EconomyType,
    pub government: GovernmentType,
    pub tech_level: u8,
    pub population: String,
    pub description: String,
    pub star: Option<StarClass>,
    pub visited: bool,
    /// One of the outer stars
    pub frontier: bool,
    pub from_here: Option<JumpFromHere>,
}

/// Recent trades and totals
#[derive(Debug, Clone)]
pub struct TradeLogView {
    pub recent: Vec<Transaction>,
    pub stats: TradeStats,
    pub best: Option<Transaction>,
}
