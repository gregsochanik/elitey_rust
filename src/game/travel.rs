use crate::data::systems::{StarSystem, SystemManager};
use crate::game::aftermath::settle;
use crate::game::combat::{Combat, Combatant};
use crate::game::fight::run_fight;
use crate::game::{Commander, Event, EventManager, EventPhase, Ship};
use crate::utils::display::{show_error, show_info, show_success};
use crate::utils::input;
use colored::*;
use rand::Rng;
use std::io;

/// Handles travel between star systems with event integration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TravelHandler {
    system_manager: SystemManager,
}

/// Fuel a jump of this distance costs, in whole light years
pub fn fuel_for_distance(distance: f64) -> u32 {
    distance.ceil() as u32
}

impl TravelHandler {
    pub fn new(system_manager: SystemManager) -> Self {
        Self { system_manager }
    }

    /// Process travel between systems with event integration
    pub fn travel_to_system(
        &self,
        commander: &mut Commander,
        ship: &mut Ship,
        event_manager: &EventManager,
        destination_name: &str,
    ) -> Result<bool, String> {
        // Validate destination (case-insensitive)
        let destination = self
            .system_manager
            .get_system_case_insensitive(destination_name)
            .ok_or_else(|| format!("System '{}' not found", destination_name))?;

        let current_system = self
            .system_manager
            .get_system_case_insensitive(&commander.location)
            .ok_or_else(|| format!("Current system '{}' not found", commander.location))?;

        if current_system.name.eq_ignore_ascii_case(&destination.name) {
            return Err("You are already in this system.".to_string());
        }

        // Calculate distance and fuel cost
        let distance = current_system.distance_to(destination);
        let fuel_cost = fuel_for_distance(distance);

        // Check if we have enough fuel
        if ship.fuel_current < fuel_cost {
            return Err(format!(
                "Insufficient fuel. Need {} light years, have {}",
                fuel_cost, ship.fuel_current
            ));
        }

        // Check if ship is too damaged
        if ship.hull_percentage == 0 {
            return Err("Ship is too damaged to travel".to_string());
        }

        // Show travel confirmation
        self.show_travel_confirmation(&commander.location, destination_name, distance);

        // Get user confirmation
        match input::get_input() {
            Ok(input) => {
                if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
                    self.process_travel_with_events(commander, ship, event_manager, destination)
                } else {
                    show_info("Travel cancelled.");
                    Ok(false)
                }
            }
            Err(_) => {
                show_error("Error reading input. Travel cancelled.");
                Ok(false)
            }
        }
    }

    /// Process the actual travel with event integration
    fn process_travel_with_events(
        &self,
        commander: &mut Commander,
        ship: &mut Ship,
        event_manager: &EventManager,
        destination: &StarSystem,
    ) -> Result<bool, String> {
        show_info("Initiating hyperspace jump...");

        let fuel_cost = self
            .jump_fuel_cost(&commander.location, destination)
            .ok_or_else(|| format!("Unknown origin system: {}", commander.location))?;

        // Phase 1: Jump Events (during hyperspace)
        let jump_event_occurred =
            self.process_jump_phase(commander, ship, event_manager, destination)?;

        ship.consume_fuel(fuel_cost);

        // Phase 2: Approach Events (after jump, before station)
        let approach_event_occurred =
            self.process_approach_phase(commander, ship, event_manager, destination)?;

        // Update location
        commander.location = destination.name.clone();

        // Show arrival message
        if jump_event_occurred || approach_event_occurred {
            show_success(&format!(
                "Arrived at {} after an eventful journey!",
                destination.name
            ));
        } else {
            show_success(&format!("Arrived safely at {}.", destination.name));
        }

        Ok(true)
    }

    fn jump_fuel_cost(&self, origin: &str, destination: &StarSystem) -> Option<u32> {
        let origin_system = self.system_manager.get_system_case_insensitive(origin)?;
        Some(fuel_for_distance(destination.distance_to(origin_system)))
    }

    /// Process jump phase events
    fn process_jump_phase(
        &self,
        commander: &mut Commander,
        ship: &mut Ship,
        event_manager: &EventManager,
        destination: &StarSystem,
    ) -> Result<bool, String> {
        if event_manager.should_trigger_event(&EventPhase::Jump, destination) {
            if let Some(event) = event_manager.select_random_event(&EventPhase::Jump, destination) {
                return self.handle_event(event, commander, ship, event_manager);
            }
        }
        Ok(false)
    }

    /// Process approach phase events
    fn process_approach_phase(
        &self,
        commander: &mut Commander,
        ship: &mut Ship,
        event_manager: &EventManager,
        destination: &StarSystem,
    ) -> Result<bool, String> {
        if event_manager.should_trigger_event(&EventPhase::Approach, destination) {
            if let Some(event) =
                event_manager.select_random_event(&EventPhase::Approach, destination)
            {
                return self.handle_event(event, commander, ship, event_manager);
            }
        }
        Ok(false)
    }

    /// Handle a specific event
    fn handle_event(
        &self,
        event: &Event,
        commander: &mut Commander,
        ship: &mut Ship,
        event_manager: &EventManager,
    ) -> Result<bool, String> {
        println!();
        println!("{}", "=".repeat(50));
        println!("{}", event.title);
        println!("{}", "=".repeat(50));
        println!("{}", event.description);
        println!();

        // Show choices
        for (i, choice) in event.choices.iter().enumerate() {
            println!("{}. {}", i + 1, choice.description);
        }
        println!();
        print!("Choose your action (1-{}): ", event.choices.len());

        // Get user choice
        match input::get_input() {
            Ok(input) => match input.trim().parse::<usize>() {
                Ok(choice_index) => {
                    if choice_index >= 1 && choice_index <= event.choices.len() {
                        let actual_index = choice_index - 1;
                        self.process_event_choice(
                            event,
                            commander,
                            ship,
                            event_manager,
                            actual_index,
                        )
                    } else {
                        show_error("Invalid choice. Defaulting to first option.");
                        self.process_event_choice(event, commander, ship, event_manager, 0)
                    }
                }
                Err(_) => {
                    show_error("Invalid input. Defaulting to first option.");
                    self.process_event_choice(event, commander, ship, event_manager, 0)
                }
            },
            Err(_) => {
                show_error("Error reading input. Defaulting to first option.");
                self.process_event_choice(event, commander, ship, event_manager, 0)
            }
        }
    }

    /// Process the chosen event outcome
    fn process_event_choice(
        &self,
        event: &Event,
        commander: &mut Commander,
        ship: &mut Ship,
        event_manager: &EventManager,
        choice_index: usize,
    ) -> Result<bool, String> {
        match event_manager.process_event(event, commander, ship, choice_index) {
            Ok(outcome) => {
                println!();
                println!("{}", outcome.message);
                println!();

                // Show updated status
                if outcome.credits_change != 0 {
                    show_info(&format!("Credits: {}", commander.credits));
                }
                if outcome.hull_damage != 0 {
                    show_info(&format!("Hull: {}", ship.hull_display()));
                }
                if outcome.fuel_change != 0 {
                    show_info(&format!("Fuel: {}", ship.fuel_current));
                }

                if outcome.starts_combat {
                    self.fight_pirates(commander, ship);
                }

                Ok(true)
            }
            Err(e) => {
                show_error(&format!("Error processing event: {}", e));
                Ok(false)
            }
        }
    }

    /// Fight a pirate raider and apply the result to the player
    fn fight_pirates(&self, commander: &mut Commander, ship: &mut Ship) {
        let mut combat = Combat::new(Combatant::from_ship(ship), Combatant::pirate_raider());
        let mut dice = rand::thread_rng();
        let mut roll = || dice.gen::<f64>();

        let result = run_fight(
            &mut combat,
            &mut io::stdin().lock(),
            &mut io::stdout(),
            &mut roll,
        );
        match result {
            Ok(outcome) => {
                println!();
                for message in settle(
                    outcome,
                    combat.player.hull,
                    commander,
                    ship,
                    &mut rand::thread_rng(),
                ) {
                    show_info(&message);
                }
            }
            Err(error) => show_error(&format!("The fight was interrupted: {}", error)),
        }
    }

    /// Show travel confirmation
    fn show_travel_confirmation(&self, from: &str, to: &str, distance: f64) {
        println!();
        println!("{}", "Travel Confirmation".cyan().bold());
        println!("{}", "===================".cyan());
        println!("From: {}", from.yellow());
        println!("To: {}", to.yellow());
        println!("Distance: {:.1} light years", distance);
        println!("{}", "Confirm travel? (y/n): ".cyan());
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
        let mut commander = Commander::new("Test".to_string());
        let mut ship = Ship::new_cobra_mk3();
        let fuel = ship.fuel_current;

        for name in ["LAVE", "lave"] {
            let result = travel_handler.travel_to_system(
                &mut commander,
                &mut ship,
                &EventManager::new(),
                name,
            );
            assert_eq!(result, Err("You are already in this system.".to_string()));
        }
        assert_eq!(ship.fuel_current, fuel);
        assert_eq!(commander.location, "LAVE");
    }

    #[test]
    fn fuel_for_distance_rounds_up_to_whole_light_years() {
        assert_eq!(fuel_for_distance(2.0), 2);
        assert_eq!(fuel_for_distance(2.3), 3);
        assert_eq!(fuel_for_distance(6.7), 7);
    }

    #[test]
    fn jump_fuel_cost_rounds_distance_up() {
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let diso = system_manager.get_system("DISO").unwrap().clone();
        let lave = system_manager.get_system("LAVE").unwrap().clone();
        let distance = diso.distance_to(&lave);
        assert!(distance > 2.0 && distance < 3.0);

        let travel_handler = TravelHandler::new(system_manager);
        assert_eq!(travel_handler.jump_fuel_cost("LAVE", &diso), Some(3));
    }

    #[test]
    fn jump_fuel_cost_is_measured_from_the_origin() {
        let system_manager = SystemManager::new().expect("Failed to load galaxy data");
        let destination = system_manager.get_system("DISO").unwrap().clone();
        let travel_handler = TravelHandler::new(system_manager);

        assert_eq!(travel_handler.jump_fuel_cost("LAVE", &destination), Some(3));
        assert_eq!(travel_handler.jump_fuel_cost("lave", &destination), Some(3));
        assert_eq!(travel_handler.jump_fuel_cost("UNKNOWN", &destination), None);
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

        // Should be able to get LAVE system info
        let lave_info = travel_handler.get_system_info("LAVE");
        assert!(lave_info.is_some());
        assert_eq!(lave_info.unwrap().name, "LAVE");
    }
}
