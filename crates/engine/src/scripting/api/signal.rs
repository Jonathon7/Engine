use crate::traffic::{SignalHead, LightPhase};

pub fn get_phase(signal_head: &SignalHead) -> LightPhase {
    signal_head.phase
}

pub fn set_phase(signal_head: &mut SignalHead, phase: LightPhase) {
    signal_head.phase = phase;
}

pub fn get_signal_set_by_name(traffic_signals: &Vec<(String, [SignalHead; 4])>, name: &str) -> Option<usize> {
    Some(traffic_signals.iter().enumerate().find(|(_, (intersection, _))| intersection == name)?.0)
}
