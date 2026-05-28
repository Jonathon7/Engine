use crate::scripting::api::signal::{get_phase, set_phase, get_signal_set_by_name};
use crate::scripting::api::time::{Timer};
use crate::traffic::{TrafficSignals, SignalHead, LightPhase};
use std::collections::HashMap;
use std::rc::{Rc};
use std::cell::RefCell;
use std::time::Duration;

pub struct ScriptingEnvironment {
    traffic_signals: Rc<RefCell<Vec<(String, [SignalHead; 4])>>>,
    signals_by_position: Rc<RefCell<HashMap<(usize, usize), (usize, usize)>>>,
    // USER SCRIPT SPACE
    timer: Timer,
    phase: Phase
}

impl ScriptingEnvironment {
    pub fn new(traffic: &TrafficSignals) -> Self {
        ScriptingEnvironment {
            traffic_signals: Rc::clone(&traffic.traffic_signals),
            signals_by_position: Rc::clone(&traffic.signals_by_position),
            timer: Timer::new(),
            phase: Phase::NsGreen
        }
    }

    pub fn start(&mut self) {
        self.timer.start();
    }

    pub fn update(&mut self) {
        let signal_set_idx: usize = get_signal_set_by_name(&self.traffic_signals.borrow(), "Main Intersection").unwrap();
        let mut signal_ref = self.traffic_signals.borrow_mut();
        let signal_set = &mut signal_ref[signal_set_idx].1;
        let light_configuration = self.phase.light_configuration();

        match light_configuration.ns {
            LightPhase::Red => {
                set_phase(&mut signal_set[0], LightPhase::Red);
                set_phase(&mut signal_set[1], LightPhase::Red);
            },
            LightPhase::Yellow => {
                set_phase(&mut signal_set[0], LightPhase::Yellow);
                set_phase(&mut signal_set[1], LightPhase::Yellow);
            },
            LightPhase::Green => {
                set_phase(&mut signal_set[0], LightPhase::Green);
                set_phase(&mut signal_set[1], LightPhase::Green);
            }
        }
       
        match light_configuration.ew {
            LightPhase::Red => {
                set_phase(&mut signal_set[2], LightPhase::Red);
                set_phase(&mut signal_set[3], LightPhase::Red);
            },
            LightPhase::Yellow => {
                set_phase(&mut signal_set[2], LightPhase::Yellow);
                set_phase(&mut signal_set[3], LightPhase::Yellow);
            },
            LightPhase::Green => {
                set_phase(&mut signal_set[2], LightPhase::Green);
                set_phase(&mut signal_set[3], LightPhase::Green);
            }
        }

        if self.phase.duration() <= self.timer.elapsed() {
            self.phase = self.phase.next();
            self.timer.reset();
            self.timer.start();
        }
    }
}

// USER CODE SPACE
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    NsGreen,
    NsYellow,
    AllRedToEw,
    EwGreen,
    EwYellow,
    AllRedToNs
}

struct LightConfiguration {
    pub ns: LightPhase,
    pub ew: LightPhase
}

impl Phase {
    pub fn light_configuration(self) -> LightConfiguration {
        use LightPhase::*;
        match self {
            Phase::NsGreen => LightConfiguration { ns: Green, ew: Red },
            Phase::NsYellow => LightConfiguration { ns: Yellow, ew: Red },
            Phase::AllRedToEw => LightConfiguration { ns: Red, ew: Red },
            Phase::EwGreen => LightConfiguration { ns: Red, ew: Green },
            Phase::EwYellow => LightConfiguration { ns: Red, ew: Yellow },
            Phase::AllRedToNs => LightConfiguration { ns: Red, ew: Red }
        }
    }    

    pub fn next(self) -> Self {
        match self {
            Phase::NsGreen => Phase::NsYellow,
            Phase::NsYellow => Phase::AllRedToEw,
            Phase::AllRedToEw => Phase::EwGreen,
            Phase::EwGreen => Phase::EwYellow,
            Phase::EwYellow => Phase::AllRedToNs,
            Phase::AllRedToNs => Phase::NsGreen
        }
    }

    pub fn duration(self) -> Duration {
        match self {
            Phase::NsGreen | Phase::EwGreen => Duration::from_secs(3),
            Phase::NsYellow | Phase::EwYellow => Duration::from_secs(1),
            Phase::AllRedToNs | Phase::AllRedToEw => Duration::from_secs(1)
        }
    }
}

