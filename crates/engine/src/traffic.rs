use crate::platform::tilemap::{Tile, IntoTileIndex, MAP_WIDTH, MAP_HEIGHT};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LightPhase {
    Red,
    Yellow,
    Green
}

#[derive(Clone, Copy)]
pub struct SignalHead {
    pub x: usize,
    pub y: usize,
    pub phase: LightPhase
}

pub struct TrafficSignals {
    pub traffic_signals: Rc<RefCell<Vec<(String, [SignalHead; 4])>>>,
    pub signals_by_position: Rc<RefCell<HashMap<(usize, usize), (usize, usize)>>>
}

impl TrafficSignals {
    pub fn new(map: &Vec<Vec<Tile>>) -> Self {
        let traffic_signal_sets: Rc<RefCell<Vec<(String, [SignalHead; 4])>>> = Rc::new(RefCell::new(Vec::new()));
        let signals_by_position_values: Rc<RefCell<HashMap<(usize, usize), (usize, usize)>>> = Rc::new(RefCell::new(HashMap::new()));
        for (y, row) in map.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if *tile == Tile::Stoplight {

                    let mut signal_sets = traffic_signal_sets.borrow_mut();
                    let mut signals_by_position = signals_by_position_values.borrow_mut();

                    let south_light_x: usize = x - 1;
                    let south_light_y: usize = y + 3;
                    
                    if x < 1 || south_light_x > MAP_WIDTH || south_light_y < 3 || south_light_y > MAP_HEIGHT { continue }
                    if map[south_light_y][south_light_x] != Tile::Stoplight { continue }

                    let north_signal_head = SignalHead { x, y, phase: LightPhase::Red };
                    let south_signal_head = SignalHead { x: south_light_x, y: south_light_y, phase: LightPhase::Red };
                    let east_signal_head = SignalHead { x: x + 1, y: y + 2, phase: LightPhase::Red };
                    let west_signal_head = SignalHead { x: x - 2, y: y + 1, phase: LightPhase::Red };

                    signals_by_position.insert((north_signal_head.x, north_signal_head.y), (signal_sets.len(), 0));
                    signals_by_position.insert((south_signal_head.x, south_signal_head.y),  (signal_sets.len(), 1));
                    signals_by_position.insert((east_signal_head.x, east_signal_head.y), (signal_sets.len(), 2));
                    signals_by_position.insert((west_signal_head.x, west_signal_head.y), (signal_sets.len(), 3));

                    signal_sets.push((String::from("Main Intersection"), [north_signal_head, south_signal_head, east_signal_head, west_signal_head]));
                }
            }
        }

        TrafficSignals {
            traffic_signals: traffic_signal_sets,
            signals_by_position: signals_by_position_values
        }
    }

    pub fn get_signal_head_by_position<T: IntoTileIndex>(&self, x: T, y: T) -> Option<SignalHead> {
        let index_x = x.into_index()?;
        let index_y = y.into_index()?;

        let &(traffic_signal_index, signal_head_index) = self.signals_by_position.borrow().get(&(index_x, index_y))?;

        Some(self.traffic_signals.borrow()[traffic_signal_index].1[signal_head_index])
    }

    pub fn get_signal_set_by_name(&self, name: &str) -> Option<[SignalHead; 4]> {
        Some(self.traffic_signals.borrow().iter().find(|(intersection, _)| intersection == name)?.1)
    }
}

