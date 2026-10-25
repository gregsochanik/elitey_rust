use crate::data::enemies::find_enemy;
use crate::data::systems::StarSystem;
use crate::game::combat::{Combat, Combatant, Outcome};
use crate::game::events::{Event, EventPhase, EventReport};
use crate::game::jobs::{self, ActiveJob};
use crate::game::smuggling::{self, Answer, Inspection, InspectionChoice};
use crate::game::travel::{redirect_target, JumpPlan, TravelError};
use crate::game::{aftermath, GameState};
use rand::Rng;
use std::collections::VecDeque;
use thiserror::Error;

/// A line the front end should show the player
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    Blank,
    Text(String),
    Info(String),
    Success(String),
    Error(String),
}

/// What a journey needs from the player next
#[derive(Debug, Clone)]
pub enum Step {
    /// Show the event and answer with `Input::Choice`
    Event(Event),
    /// Play the fight and answer with `Input::FightEnded` (or `FightInterrupted`)
    Fight(Combat),
    /// A patrol has stopped a smuggler; answer with `Input::Inspection`
    Inspection { bribe_cost: u32, can_bribe: bool },
    /// The journey is over
    Arrived {
        arrived_at: String,
        planned: String,
        eventful: bool,
    },
}

/// The player's answer to a `Step`
#[derive(Debug, Clone)]
pub enum Input {
    /// Begin, or carry on after notices
    Continue,
    /// The 0-based event choice
    Choice(usize),
    Inspection(InspectionChoice),
    FightEnded(Combat),
    FightInterrupted(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum JourneyError {
    #[error("That is not one of the choices")]
    InvalidChoice,
    #[error("The journey is waiting for a different answer")]
    WrongInput,
    #[error("The journey is over")]
    Finished,
}

#[derive(Debug, Clone)]
enum Stage {
    Start,
    JumpEvent,
    AwaitEvent { event: Event, phase: EventPhase },
    AwaitEventFight { phase: EventPhase },
    Fuel,
    ApproachEvent,
    Arrive,
    AwaitInspection { jobs: Vec<ActiveJob> },
    AwaitPatrolFight { jobs: Vec<ActiveJob> },
    NextBounty,
    AwaitBountyFight { entry: ActiveJob },
    Done,
}

/// A jump in progress: events, fights and arrival, one step at a time
#[derive(Debug, Clone)]
pub struct Journey {
    plan: JumpPlan,
    stage: Stage,
    eventful: bool,
    landed: Option<String>,
    bounties: VecDeque<ActiveJob>,
    hunt_over: bool,
    notices: Vec<Notice>,
}

impl Journey {
    fn new(plan: JumpPlan, stage: Stage) -> Self {
        Self {
            plan,
            stage,
            eventful: false,
            landed: None,
            bounties: VecDeque::new(),
            hunt_over: false,
            notices: Vec::new(),
        }
    }

    /// Lines to show since the last time they were taken
    pub fn take_notices(&mut self) -> Vec<Notice> {
        std::mem::take(&mut self.notices)
    }

    fn text(&mut self, messages: Vec<String>) {
        self.notices.extend(messages.into_iter().map(Notice::Text));
    }

    fn info(&mut self, message: impl Into<String>) {
        self.notices.push(Notice::Info(message.into()));
    }
}

fn fight_result(input: Input) -> Result<Result<(Outcome, u32), String>, JourneyError> {
    match input {
        Input::FightEnded(combat) => Ok(Ok((combat.outcome(), combat.player.hull))),
        Input::FightInterrupted(error) => Ok(Err(error)),
        _ => Err(JourneyError::WrongInput),
    }
}

impl GameState {
    /// Check a jump can be made and say what it would cost
    pub fn plan_jump(&self, destination: &str) -> Result<JumpPlan, TravelError> {
        self.travel_handler
            .plan_jump(&self.commander, &self.ship, destination)
    }

    /// Start a planned jump; the first step is returned with the journey
    pub fn start_jump(&mut self, plan: JumpPlan, rng: &mut impl Rng) -> (Journey, Step) {
        let mut journey = Journey::new(plan, Stage::Start);
        let step = self
            .advance_journey(&mut journey, Input::Continue, rng)
            .expect("a new journey accepts Continue");
        (journey, step)
    }

    /// Answer the current step and move on to the next
    pub fn advance_journey(
        &mut self,
        journey: &mut Journey,
        input: Input,
        rng: &mut impl Rng,
    ) -> Result<Step, JourneyError> {
        let mut input = Some(input);
        loop {
            match std::mem::replace(&mut journey.stage, Stage::Done) {
                Stage::Start => {
                    journey.info("Initiating hyperspace jump...");
                    journey.stage = Stage::JumpEvent;
                }
                Stage::JumpEvent => {
                    let planned = self.system_named(&journey.plan.destination);
                    journey.stage = Stage::Fuel;
                    if let Some(event) = self.roll_event(&EventPhase::Jump, &planned, rng) {
                        journey.eventful = true;
                        journey.stage = Stage::AwaitEvent {
                            event: event.clone(),
                            phase: EventPhase::Jump,
                        };
                        return Ok(Step::Event(event));
                    }
                }
                Stage::AwaitEvent { event, phase } => {
                    let Some(Input::Choice(choice)) = input.take() else {
                        journey.stage = Stage::AwaitEvent { event, phase };
                        return Err(JourneyError::WrongInput);
                    };
                    let report = match self.event_manager.process_event_reported(
                        &event,
                        &mut self.commander,
                        &mut self.ship,
                        choice,
                        rng,
                    ) {
                        Ok(report) => report,
                        Err(_) => {
                            journey.stage = Stage::AwaitEvent { event, phase };
                            return Err(JourneyError::InvalidChoice);
                        }
                    };
                    let starts_combat = self.report_event(journey, &report, rng);
                    if starts_combat {
                        journey.stage = Stage::AwaitEventFight { phase };
                        return Ok(Step::Fight(self.fight_against("pirate-raider")));
                    }
                    journey.stage = after_event(&phase);
                }
                Stage::AwaitEventFight { phase } => {
                    let result = match input.take().map(fight_result) {
                        Some(Ok(result)) => result,
                        Some(Err(error)) => {
                            journey.stage = Stage::AwaitEventFight { phase };
                            return Err(error);
                        }
                        None => return Err(JourneyError::WrongInput),
                    };
                    match result {
                        Ok((outcome, hull)) => {
                            journey.notices.push(Notice::Blank);
                            for message in aftermath::settle(
                                outcome,
                                hull,
                                &mut self.commander,
                                &mut self.ship,
                                rng,
                            ) {
                                journey.info(message);
                            }
                        }
                        Err(error) => journey.notices.push(Notice::Error(format!(
                            "The fight was interrupted: {}",
                            error
                        ))),
                    }
                    journey.stage = after_event(&phase);
                }
                Stage::Fuel => {
                    self.ship.consume_fuel(journey.plan.fuel_cost);
                    if self.commander.location != journey.plan.origin {
                        journey.landed = Some(self.commander.location.clone());
                    }
                    journey.stage = Stage::ApproachEvent;
                }
                Stage::ApproachEvent => {
                    let arrival = self
                        .system_named(journey.landed.as_ref().unwrap_or(&journey.plan.destination));
                    journey.stage = Stage::Arrive;
                    if let Some(event) = self.roll_event(&EventPhase::Approach, &arrival, rng) {
                        journey.eventful = true;
                        journey.stage = Stage::AwaitEvent {
                            event: event.clone(),
                            phase: EventPhase::Approach,
                        };
                        return Ok(Step::Event(event));
                    }
                }
                Stage::Arrive => {
                    let arrived_at = journey
                        .landed
                        .clone()
                        .unwrap_or_else(|| journey.plan.destination.clone());
                    self.commander.location = arrived_at.clone();
                    if arrived_at != journey.plan.destination {
                        journey.info(format!(
                            "You emerge at {}, not {}.",
                            arrived_at, journey.plan.destination
                        ));
                    }
                    journey.notices.push(Notice::Success(if journey.eventful {
                        format!("Arrived at {} after an eventful journey!", arrived_at)
                    } else {
                        format!("Arrived safely at {}.", arrived_at)
                    }));
                    self.world_clock += 1;
                    self.job_board.clear();
                    if let Some(step) = self.settle_arrival_jobs(journey, rng) {
                        return Ok(step);
                    }
                }
                Stage::AwaitInspection { jobs } => {
                    let Some(Input::Inspection(choice)) = input.take() else {
                        journey.stage = Stage::AwaitInspection { jobs };
                        return Err(JourneyError::WrongInput);
                    };
                    match smuggling::answer_inspection(
                        &jobs,
                        choice,
                        &mut self.commander,
                        &mut self.ship,
                    ) {
                        Answer::Settled(messages) => {
                            journey.text(messages);
                            journey.stage = Stage::NextBounty;
                        }
                        Answer::Fight => {
                            journey.stage = Stage::AwaitPatrolFight { jobs };
                            return Ok(Step::Fight(self.fight_against("patrol-vessel")));
                        }
                    }
                }
                Stage::AwaitPatrolFight { jobs } => {
                    let result = match input.take().map(fight_result) {
                        Some(Ok(result)) => result,
                        Some(Err(error)) => {
                            journey.stage = Stage::AwaitPatrolFight { jobs };
                            return Err(error);
                        }
                        None => return Err(JourneyError::WrongInput),
                    };
                    let messages = smuggling::finish_patrol_fight(
                        &jobs,
                        result,
                        &mut self.commander,
                        &mut self.ship,
                        rng,
                    );
                    journey.text(messages);
                    journey.stage = Stage::NextBounty;
                }
                Stage::NextBounty => match journey.bounties.pop_front() {
                    None => journey.stage = Stage::Done,
                    Some(entry) if journey.hunt_over => {
                        let messages = jobs::fail_bounty(&entry, &mut self.commander);
                        journey.text(messages);
                        journey.stage = Stage::NextBounty;
                    }
                    Some(entry) => {
                        journey.stage = Stage::AwaitBountyFight { entry };
                        return Ok(Step::Fight(self.fight_against("bounty-target")));
                    }
                },
                Stage::AwaitBountyFight { entry } => {
                    let result = match input.take().map(fight_result) {
                        Some(Ok(result)) => result,
                        Some(Err(error)) => {
                            journey.stage = Stage::AwaitBountyFight { entry };
                            return Err(error);
                        }
                        None => return Err(JourneyError::WrongInput),
                    };
                    let (messages, goes_on) = jobs::finish_bounty_fight(
                        &entry,
                        result,
                        &mut self.commander,
                        &mut self.ship,
                        rng,
                    );
                    journey.text(messages);
                    journey.hunt_over = !goes_on;
                    journey.stage = Stage::NextBounty;
                }
                Stage::Done => {
                    journey.stage = Stage::Done;
                    return Ok(Step::Arrived {
                        arrived_at: self.commander.location.clone(),
                        planned: journey.plan.destination.clone(),
                        eventful: journey.eventful,
                    });
                }
            }
        }
    }

    fn system_named(&self, name: &str) -> StarSystem {
        self.travel_handler
            .get_system_info(name)
            .expect("journeys only visit known systems")
            .clone()
    }

    fn roll_event(
        &self,
        phase: &EventPhase,
        destination: &StarSystem,
        rng: &mut impl Rng,
    ) -> Option<Event> {
        if !self
            .event_manager
            .should_trigger_event_with(phase, destination, rng)
        {
            return None;
        }
        self.event_manager
            .pick_event(phase, destination, self.commander.standing(), rng)
            .cloned()
    }

    fn fight_against(&self, enemy_id: &str) -> Combat {
        let enemy = find_enemy(enemy_id).expect("enemy is in data/enemies.json");
        Combat::against(Combatant::from_ship(&self.ship), enemy)
    }

    /// Turn an applied event outcome into notices and apply any redirect; true when it starts a fight
    fn report_event(
        &mut self,
        journey: &mut Journey,
        report: &EventReport,
        rng: &mut impl Rng,
    ) -> bool {
        if let Some(lost) = &report.cargo_lost {
            if lost.is_empty() {
                journey.info("You had no cargo to lose.");
            }
            for (commodity, tons) in lost {
                journey.info(format!("Lost {} tons of {}", tons, commodity.name()));
            }
        }
        if let Some((commodity, tons)) = &report.cargo_gained {
            journey.info(format!("Gained {} tons of {}", tons, commodity.name()));
        }
        if report.no_room_for_cargo {
            journey.info("No room in your hold for the cargo.");
        }
        if let Some(standing) = report.new_standing {
            journey.info(format!("Your standing is now {}.", standing.name()));
        }

        let outcome = &report.outcome;
        journey.notices.push(Notice::Blank);
        journey.notices.push(Notice::Text(outcome.message.clone()));
        journey.notices.push(Notice::Blank);
        if outcome.credits_change != 0 {
            journey.info(format!("Credits: {}", self.commander.credits));
        }
        if outcome.hull_damage != 0 {
            journey.info(format!("Hull: {}", self.ship.hull_display()));
        }
        if outcome.fuel_change != 0 {
            journey.info(format!("Fuel: {}", self.ship.fuel_current));
        }

        if let Some(redirect) = outcome.redirect {
            let planned = self.system_named(&journey.plan.destination);
            let systems = self.travel_handler.get_available_systems();
            if let Some(target) =
                redirect_target(redirect, &systems, &self.commander.location, &planned, rng)
            {
                self.commander.location = target;
            }
        }
        outcome.starts_combat
    }

    /// Deliver, inspect, fight or fail the jobs that fall due here.
    /// Returns the step that needs the player, or `None` when everything settled.
    fn settle_arrival_jobs(&mut self, journey: &mut Journey, rng: &mut impl Rng) -> Option<Step> {
        let arrived_at = self.commander.location.clone();
        let arrival = jobs::settle_arrival(
            &mut self.active_jobs,
            &mut self.commander,
            &mut self.ship,
            &arrived_at,
        );
        journey.text(arrival.messages);
        journey.bounties = arrival.bounties.into();
        journey.stage = Stage::NextBounty;

        if arrival.smuggling.is_empty() {
            return None;
        }
        let government = self
            .travel_handler
            .get_system_info(&arrived_at)
            .map(|system| system.government.clone())?;
        match smuggling::inspect(
            &arrival.smuggling,
            &mut self.commander,
            &mut self.ship,
            &government,
            rng,
        ) {
            Inspection::Passed(messages) => {
                journey.text(messages);
                None
            }
            Inspection::Stopped {
                bribe_cost,
                can_bribe,
                messages,
            } => {
                journey.text(messages);
                journey.stage = Stage::AwaitInspection {
                    jobs: arrival.smuggling,
                };
                Some(Step::Inspection {
                    bribe_cost,
                    can_bribe,
                })
            }
        }
    }

    /// Play every fight automatically and hand over contraband when inspected
    pub fn finish_journey_automatically(
        &mut self,
        journey: &mut Journey,
        mut step: Step,
        rng: &mut impl Rng,
    ) -> Step {
        loop {
            let input = match step {
                Step::Event(_) => Input::Choice(0),
                Step::Fight(mut combat) => {
                    auto_fight(&mut combat, rng);
                    Input::FightEnded(combat)
                }
                Step::Inspection { .. } => Input::Inspection(InspectionChoice::HandOver),
                arrived @ Step::Arrived { .. } => return arrived,
            };
            step = self
                .advance_journey(journey, input, rng)
                .expect("automatic answers are always valid");
        }
    }

    /// Settle the jobs due at the commander's current location without any events
    pub fn settle_jobs(&mut self) {
        self.settle_jobs_with(&mut rand::thread_rng());
    }

    pub fn settle_jobs_with(&mut self, rng: &mut impl Rng) {
        let plan = JumpPlan {
            origin: self.commander.location.clone(),
            destination: self.commander.location.clone(),
            distance: 0.0,
            fuel_cost: 0,
        };
        let mut journey = Journey::new(plan, Stage::Done);
        let step = match self.settle_arrival_jobs(&mut journey, rng) {
            Some(step) => step,
            None => self
                .advance_journey(&mut journey, Input::Continue, rng)
                .expect("settling needs no answer"),
        };
        self.finish_journey_automatically(&mut journey, step, rng);
    }
}

fn after_event(phase: &EventPhase) -> Stage {
    match phase {
        EventPhase::Jump => Stage::Fuel,
        EventPhase::Approach => Stage::Arrive,
    }
}

/// Attack until the fight ends; with no weapons, surrender
pub fn auto_fight(combat: &mut Combat, rng: &mut impl Rng) {
    use crate::game::combat::Action;
    let mut roll = || rng.gen::<f64>();
    while combat.outcome() == Outcome::Ongoing {
        let action = if combat.player.weapons.is_empty() {
            Action::Surrender
        } else {
            Action::Attack
        };
        if combat.resolve_round(action, &mut roll).is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests;
