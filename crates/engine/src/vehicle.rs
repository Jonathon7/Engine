use crate::platform::tilemap::{Tile, Tilemap, TILE_SIZE};
use crate::traffic::{TrafficSignals, SignalHead, LightPhase};
use sdl3::pixels::Color;

const MAX_VEHICLE_LIMIT: usize = 1;

pub struct Vehicles {
    pub inventory: Vec<Vehicle>,
    spawn_points: Vec<(usize, usize)>
}

impl Vehicles {
    pub fn new(map: &Vec<Vec<Tile>>) -> Self {
        let mut points = Vec::new();
        for (x, row) in map.iter().enumerate() {
            for (y, tile) in row.iter().enumerate() {
                if *tile == Tile::RoadVSpawn {
                    points.push((x, y));
                }
            }
        }

        Vehicles {
            inventory: Vec::new(),
            spawn_points: points
        }
    }

    pub fn spawn_vehicle(&mut self) {
        if self.inventory.len() < MAX_VEHICLE_LIMIT && self.spawn_points.len() > 0 {
            let (y, x) = self.spawn_points[0];
            self.inventory.push(Vehicle { x: x as f32 * TILE_SIZE + 8.0, y: y as f32 * TILE_SIZE, velocity_x: 0.0, velocity_y: -50.0, color: Color::RGB(150, 150, 145) });
        }
    }

    pub fn update(&mut self, traffic_signals: &TrafficSignals, tile_map: &Tilemap, delta_time: f32) {
        self.inventory.retain(|v| tile_map.is_in_bounds(v.x, v.y));
        for vehicle in &mut self.inventory {
            let x_check = if vehicle.is_vertical() { vehicle.x } else { vehicle.x - (TILE_SIZE * 2.0) };
            let y_check = if vehicle.is_vertical() { vehicle.y - (TILE_SIZE * 2.0) } else { vehicle.y };
            let signal_head: Option<&SignalHead> = traffic_signals.get_signal_head(x_check, y_check);
            match signal_head {
                Some(val) => {
                    if val.phase == LightPhase::Red {
                        continue
                    } else {
                        vehicle.update(delta_time)
                    }
                },
                None => vehicle.update(delta_time)
            }
        }
    }
}

pub struct Vehicle {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub color: Color
}

impl Vehicle {
    pub fn update(&mut self, delta_time: f32) {
        self.x += self.velocity_x * delta_time;
        self.y += self.velocity_y * delta_time;
    }

    pub fn is_vertical(&self) -> bool {
        self.velocity_y.abs() > self.velocity_x.abs()
    }
}
