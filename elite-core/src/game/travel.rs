use crate::data::systems::{StarSystem, SystemManager};
use crate::game::events::Redirect;
use crate::game::{Commander, Ship};
use rand::Rng;
use thiserror::Error;

/// Handles travel between star systems
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TravelHandler {
    system_manager: SystemManager,
}

/// Fuel a jump of this distance costs, in whole light years
pub fn fuel_for_distance(distance: f64) -> u32 {
    distance.ceil() as u32
}

/// Why a jump cannot be made
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TravelError {
    #[error("System '{0}' not found")]
    UnknownDestination(String),
    #[error("Current system '{0}' not found")]
    UnknownOrigin(String),
    #[error("You are already in this system.")]
    AlreadyThere,
    #[error("Insufficient fuel. Need {needed} light years, have {have}")]
    NotEnoughFuel { needed: u32, have: u32 },
    #[error("Ship is too damaged to travel")]
    TooDamaged,
}

/// A jump that can be made, ready to be confirmed
#[derive(Debug, Clone, PartialEq)]
pub struct JumpPlan {
    pub origin: String,
    pub destination: String,
    pub distance: f64,
    pub fuel_cost: u32,
}

const NEAR_DESTINATION_SYSTEMS: usize = 3;

/// The system a redirecting event drops the ship at, never the origin or planned destination
pub fn redirect_target<R: Rng>(
    redirect: Redirect,
    systems: &[&StarSystem],
    origin: &str,
    destination: &StarSystem,
    rng: &mut R,
) -> Option<String> {
    let mut candidates: Vec<&StarSystem> = systems
        .iter()
        .copied()
        .filter(|system| {
            !system.name.eq_ignore_ascii_case(origin)
                && !system.name.eq_ignore_ascii_case(&destination.name)
        })
        .collect();
    candidates.sort_by(|a, b| a.name.cmp(&b.name));
    if redirect == Redirect::NearDestination {
        candidates.sort_by(|a, b| {
            destination
                .distance_to(a)
                .total_cmp(&destination.distance_to(b))
        });
        candidates.truncate(NEAR_DESTINATION_SYSTEMS);
    }
    let pick = rng.gen_range(0..candidates.len().max(1));
    candidates.get(pick).map(|system| system.name.clone())
}

impl TravelHandler {
    pub fn new(system_manager: SystemManager) -> Self {
        Self { system_manager }
    }

    /// Check a jump to `destination_name` (any case) can be made and say what it would cost
    pub fn plan_jump(
        &self,
        commander: &Commander,
        ship: &Ship,
        destination_name: &str,
    ) -> Result<JumpPlan, TravelError> {
        let destination = self
            .system_manager
            .get_system_case_insensitive(destination_name)
            .ok_or_else(|| TravelError::UnknownDestination(destination_name.to_string()))?;
        let current_system = self
            .system_manager
            .get_system_case_insensitive(&commander.location)
            .ok_or_else(|| TravelError::UnknownOrigin(commander.location.clone()))?;

        if current_system.name.eq_ignore_ascii_case(&destination.name) {
            return Err(TravelError::AlreadyThere);
        }

        let distance = current_system.distance_to(destination);
        let fuel_cost = fuel_for_distance(distance);
        if ship.fuel_current < fuel_cost {
            return Err(TravelError::NotEnoughFuel {
                needed: fuel_cost,
                have: ship.fuel_current,
            });
        }
        if ship.hull_percentage == 0 {
            return Err(TravelError::TooDamaged);
        }

        Ok(JumpPlan {
            origin: current_system.name.clone(),
            destination: destination.name.clone(),
            distance,
            fuel_cost,
        })
    }

    /// Get available systems for travel
    pub fn get_available_systems(&self) -> Vec<&StarSystem> {
        self.system_manager.get_all_systems()
    }

    /// Get system information (case-insensitive)
    pub fn get_system_info(&self, system_name: &str) -> Option<&StarSystem> {
        self.system_manager.get_system_case_insensitive(system_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jumping_to_the_current_system_is_refused_without_cost() {
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let travel_handler = TravelHandler::new(system_manager);
        let commander = Commander::new("Test".to_string());
        let ship = Ship::new_cobra_mk3();
        let fuel = ship.fuel_current;

        for name in ["Sol", "sol"] {
            let result = travel_handler.plan_jump(&commander, &ship, name);
            assert_eq!(result, Err(TravelError::AlreadyThere));
        }
        assert_eq!(ship.fuel_current, fuel);
        assert_eq!(commander.location, "Sol");
    }

    #[test]
    fn fuel_for_distance_rounds_up_to_whole_light_years() {
        assert_eq!(fuel_for_distance(2.0), 2);
        assert_eq!(fuel_for_distance(2.3), 3);
        assert_eq!(fuel_for_distance(6.7), 7);
    }

    #[test]
    fn a_plan_rounds_the_distance_up_to_whole_light_years() {
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let proxima = system_manager
            .get_system("Proxima Centauri")
            .unwrap()
            .clone();
        let sol = system_manager.get_system("Sol").unwrap().clone();
        let distance = proxima.distance_to(&sol);
        assert!(distance > 2.0 && distance < 3.0);

        let travel_handler = TravelHandler::new(system_manager);
        let commander = Commander::new("Test".to_string());
        let ship = Ship::new_cobra_mk3();
        let plan = travel_handler
            .plan_jump(&commander, &ship, "proxima centauri")
            .unwrap();

        assert_eq!(plan.origin, "Sol");
        assert_eq!(plan.destination, "Proxima Centauri");
        assert_eq!(plan.fuel_cost, 3);
        assert!((plan.distance - distance).abs() < 1e-9);
    }

    #[test]
    fn a_plan_is_refused_for_unknown_systems_low_fuel_and_a_wrecked_ship() {
        let travel_handler =
            TravelHandler::new(SystemManager::new().expect("Failed to load galaxy data"));
        let commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();

        assert_eq!(
            travel_handler.plan_jump(&commander, &ship, "NOWHERE"),
            Err(TravelError::UnknownDestination("NOWHERE".to_string()))
        );

        ship.fuel_current = 1;
        assert_eq!(
            travel_handler.plan_jump(&commander, &ship, "Proxima Centauri"),
            Err(TravelError::NotEnoughFuel { needed: 3, have: 1 })
        );

        let mut wrecked = Ship::new_cobra_mk3();
        wrecked.hull_percentage = 0;
        assert_eq!(
            travel_handler.plan_jump(&commander, &wrecked, "Proxima Centauri"),
            Err(TravelError::TooDamaged)
        );
    }

    #[test]
    fn test_travel_handler_creation() {
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let travel_handler = TravelHandler::new(system_manager);

        // Should have access to systems
        let systems = travel_handler.get_available_systems();
        assert!(!systems.is_empty());
    }

    #[test]
    fn test_get_system_info() {
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let travel_handler = TravelHandler::new(system_manager);

        // Should be able to get Sol system info
        let lave_info = travel_handler.get_system_info("Sol");
        assert!(lave_info.is_some());
        assert_eq!(lave_info.unwrap().name, "Sol");
    }

    fn systems_for_redirects() -> SystemManager {
        SystemManager::new().expect("Failed to load galaxy data")
    }

    #[test]
    fn a_redirect_never_lands_at_the_origin_or_the_planned_destination() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let manager = systems_for_redirects();
        let systems = manager.get_all_systems();
        let destination = manager.get_system("Proxima Centauri").unwrap();
        for redirect in [Redirect::Anywhere, Redirect::NearDestination] {
            for seed in 0..200 {
                let target = redirect_target(
                    redirect,
                    &systems,
                    "Sol",
                    destination,
                    &mut StdRng::seed_from_u64(seed),
                )
                .unwrap();
                assert!(target != "Sol" && target != "Proxima Centauri");
            }
        }
    }

    #[test]
    fn an_anywhere_redirect_reaches_distant_systems() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let manager = systems_for_redirects();
        let systems = manager.get_all_systems();
        let destination = manager.get_system("Proxima Centauri").unwrap();
        let targets: std::collections::HashSet<String> = (0..300)
            .filter_map(|seed| {
                redirect_target(
                    Redirect::Anywhere,
                    &systems,
                    "Sol",
                    destination,
                    &mut StdRng::seed_from_u64(seed),
                )
            })
            .collect();
        assert!(targets.len() > NEAR_DESTINATION_SYSTEMS);
    }

    #[test]
    fn a_near_destination_redirect_lands_at_one_of_the_closest_systems() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let manager = systems_for_redirects();
        let systems = manager.get_all_systems();
        let destination = manager.get_system("Proxima Centauri").unwrap();
        let mut closest: Vec<&StarSystem> = systems
            .iter()
            .copied()
            .filter(|system| system.name != "Sol" && system.name != "Proxima Centauri")
            .collect();
        closest.sort_by(|a, b| {
            destination
                .distance_to(a)
                .total_cmp(&destination.distance_to(b))
        });
        let allowed: Vec<&str> = closest
            .iter()
            .take(NEAR_DESTINATION_SYSTEMS)
            .map(|system| system.name.as_str())
            .collect();
        for seed in 0..200 {
            let target = redirect_target(
                Redirect::NearDestination,
                &systems,
                "Sol",
                destination,
                &mut StdRng::seed_from_u64(seed),
            )
            .unwrap();
            assert!(allowed.contains(&target.as_str()), "{target}");
        }
    }
}
