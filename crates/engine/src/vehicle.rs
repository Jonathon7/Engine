use crate::platform::tilemap::{Tile, Tilemap, TILE_SIZE, MAP_DATA, MAP_HEIGHT, MAP_WIDTH, IntoTileIndex};
use crate::traffic::{TrafficSignals, SignalHead, LightPhase};
use crate::pathfinding::{Point, astar};
use rand::{random_range, rng};
use rand::seq::SliceRandom;
use sdl3::pixels::Color;

const MAX_VEHICLE_LIMIT: usize = 15;
const VEHICLE_FOLLOW_DISTANCE: f32 = TILE_SIZE;
pub const VEHICLE_WIDTH: f32 = 16.0;
pub const VEHICLE_HEIGHT: f32 = 24.0;

#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Null
}

enum TurnType {
    Straight,
    Right,
    Left
}

pub struct Vehicles {
    pub inventory: Vec<Vehicle>,
    spawn_points: Vec<((usize, usize), Tile)>,
    pathfinding_map: Vec<Vec<bool>>,
    pub char_map: Vec<Vec<char>>
}

impl Vehicles {
    pub fn new(map: &Vec<Vec<Tile>>) -> Self {
        let mut points = Vec::new();
        let mut pathfinding_map: Vec<Vec<bool>> = Vec::new();
        for (y, row) in map.iter().enumerate() {
            let mut map_row: Vec<bool> = Vec::new();
            for (x, tile) in row.iter().enumerate() {
                if *tile == Tile::RoadVNSpawn ||
                   *tile == Tile::RoadVSSpawn ||
                   *tile == Tile::RoadHESpawn ||
                   *tile == Tile::RoadHWSpawn {
                    points.push(((y, x), *tile));
                    map_row.push(true);
                } else if *tile == Tile::RoadH ||
                          *tile == Tile::RoadV ||
                          *tile == Tile::RoadIntersect ||
                          *tile == Tile::Stoplight {
                    map_row.push(true);
                } else {
                    map_row.push(false);
                }
            }
            pathfinding_map.push(map_row);
        }

        let char_map: Vec<Vec<char>> = MAP_DATA.lines()
            .filter(|l| !l.is_empty())
            .map(|line| line.chars().collect())
            .collect();
                                    

        Vehicles {
            inventory: Vec::new(),
            spawn_points: points,
            pathfinding_map,
            char_map
        }
    }

    pub fn spawn_vehicle(&mut self) {
        if self.inventory.len() < MAX_VEHICLE_LIMIT && self.spawn_points.len() > 0 {
            let idx: usize = random_range(0..self.spawn_points.len());
            let (y, x) = self.spawn_points[idx].0;
            let mut angle = 0.0;
            let x_pos_offset = TILE_SIZE / 2.0;
            let y_pos_offset = TILE_SIZE / 2.0;

            match self.spawn_points[idx].1 {
                Tile::RoadVNSpawn => angle = 0.0,
                Tile::RoadVSSpawn => angle = 180.0,
                Tile::RoadHESpawn => angle = 90.0,
                Tile::RoadHWSpawn => angle = 270.0,
                _ => return,
            };

            let mut vehicle = Vehicle { x: x as f32 * TILE_SIZE + x_pos_offset, y: y as f32 * TILE_SIZE + y_pos_offset, facing_angle: angle, speed: 50.0, color: Color::RGB(150, 150, 145), path: None, path_idx: 0 };
            let destination = self.random_destination(&vehicle).unwrap();
            let path = astar(&self.pathfinding_map, Point { x: x.try_into().unwrap(), y: y.try_into().unwrap() }, Point { x: destination.0.try_into().unwrap(), y: destination.1.try_into().unwrap() });
            vehicle.set_path(self.smooth_path(self.normalize_vehicle_path(&path.unwrap(), &vehicle), 12));
            self.inventory.push(vehicle);
        }
    }

    fn char_to_direction(&self, c: char) -> Option<Direction> {
        match c {
            '^' | '}' => Some(Direction::North),
            'v' | '{' => Some(Direction::South),
            '>' | '[' => Some(Direction::East),
            '<' | ']' => Some(Direction::West),
            _ => None,
        }
    }

    fn direction_delta(&self, dir: Direction) -> (i32, i32) {
        match dir {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East  => (1, 0),
            Direction::West  => (-1, 0),
            Direction::Null => (0, 0)
        }
    }

    fn is_intersection_tile(&self, c: char) -> bool {
        c == '+'
    }

    fn turn_type(&self, entry: &Direction, exit: &Direction) -> TurnType {
        if entry == exit { return TurnType::Straight; }

        let is_right = matches!(
            (entry, exit),
            (Direction::North, Direction::East)
            | (Direction::East, Direction::South)
            | (Direction::South, Direction::West)
            | (Direction::West, Direction::North)
        );
        if is_right { TurnType::Right } else { TurnType::Left }
    }

    fn find_exit_direction(&self, path: &[Point]) -> Option<Direction> {
        let last_zone_idx = path.iter()
            .rposition(|p| self.is_intersection_tile(self.char_map[p.y as usize][p.x as usize]))?;

        let after = path.get(last_zone_idx + 1)?;
        let last = &path[last_zone_idx];

        let dx = after.x - last.x;
        let dy = after.y - last.y;

        match (dx.signum(), dy.signum()) {
            ( 1,  0) => Some(Direction::East),
            (-1,  0) => Some(Direction::West),
            ( 0, -1) => Some(Direction::North),
            ( 0,  1) => Some(Direction::South),
            _ => None,
        }
    }

    fn normalize_vehicle_path(&self, path: &Vec<Point>, vehicle: &Vehicle) -> Vec<Point> {
        if path.len() < 2 {
            return path.clone();
        }

        let mut normalized_path: Vec<Point> = Vec::new();
        let mut pos = path[0].clone();
        let dest = path.last().unwrap();
        let mut dir = vehicle.direction();

        normalized_path.push(pos.clone());

        while pos != *dest {
            let (dx, dy) = self.direction_delta(dir);
            let next = Point { x: pos.x + dx, y: pos.y + dy };
            let next_y = next.y as usize;
            let next_x = next.x as usize;

            if next_y >= MAP_HEIGHT || next_x >= MAP_WIDTH {
                break;
            }

            let c = self.char_map[next_y][next_x];

            if !self.is_intersection_tile(c) {
                pos = next;
                normalized_path.push(pos.clone());
            } else {
                let exit_dir = self.find_exit_direction(path.as_slice()).unwrap();
                let turn_type = self.turn_type(&dir, &exit_dir);

                let mut zone_dir = dir.clone();
                let mut intersection_tile_count = 0;

                pos = next;
                normalized_path.push(pos.clone());

                loop {
                    let c = self.char_map[pos.y as usize][pos.x as usize];
                    if self.is_intersection_tile(c) {
                        intersection_tile_count += 1;
                    }

                    match turn_type {
                        TurnType::Right if self.is_intersection_tile(c) && intersection_tile_count == 1 => {
                            zone_dir = exit_dir.clone();
                        },
                        TurnType::Left if self.is_intersection_tile(c) && intersection_tile_count == 2 => {
                            zone_dir = exit_dir.clone();
                        },
                        _ => {}
                    }
    
                    let (zdx, zdy) = self.direction_delta(zone_dir);
                    let ahead = Point { x: pos.x + zdx, y: pos.y + zdy };

                    let ahead_y = (pos.y as usize).into_index().unwrap();
                    let ahead_x = (pos.x as usize).into_index().unwrap();
                    let ahead_c = self.char_map[ahead_y][ahead_x];

                    if !self.is_intersection_tile(ahead_c) {
                        break;
                    }

                    pos = ahead;
                    normalized_path.push(pos.clone());
                }

                dir = exit_dir;
            }
        }

        normalized_path
    }
    
    fn smooth_path(&mut self, path: Vec<Point>, segments: usize) -> Vec<Point> {
        let mut smoothed_path: Vec<Point> = Vec::new();

        let to_px = |p: &Point| -> Point {
            Point {
                x: (p.x as f32 * TILE_SIZE + TILE_SIZE * 0.5) as i32,
                y: (p.y as f32 * TILE_SIZE + TILE_SIZE * 0.5) as i32
            }
        };

        let mut i = 0;
        while i < path.len() {
            if i == 0 || i == path.len() - 1 {
                smoothed_path.push(to_px(&path[i].clone()));
                i += 1;
                continue;
            }

            let prev = to_px(&path[i - 1]);
            let mut curr = to_px(&path[i]);
            let next = to_px(&path[i + 1]);

            let dx1 = (curr.x - prev.x) as f32;
            let dy1 = (curr.y - prev.y) as f32;
            let dx2 = (next.x - curr.x) as f32;
            let dy2 = (next.y - curr.y) as f32;

            let is_turn = (dx1.abs() > 0.01 && dy2.abs() > 0.01)
                       || (dy1.abs() > 0.01 && dx2.abs() > 0.01);

            if is_turn {
                let p0 = ((prev.x + curr.x) as f32 * 0.5, (prev.y + curr.y) as f32 * 0.5);
                let p2 = ((curr.x + next.x) as f32 * 0.5, (curr.y + next.y) as f32 * 0.5);

                let turn_cross = dx1 * dy2 - dy1 * dx2;
                if turn_cross > 0.0 {
                    let offset = TILE_SIZE * 0.2;
                    let curr_offset = (
                        curr.x as f32 + (dx2.signum() + dx1.signum()) * offset,
                        curr.y as f32 + (dy2.signum() + dy1.signum()) * offset,
                    );
                    curr.x = curr_offset.0 as i32;
                    curr.y = curr_offset.1 as i32;
                }

                for s in 0..=segments {
                    let t = s as f32 / segments as f32;
                    let (x, y) = self.quadratic_bezier(p0, (curr.x as f32, curr.y as f32), p2, t);
                    smoothed_path.push(Point { x: x as i32, y: y as i32 });
                }
            } else {
                smoothed_path.push(curr);
            }

            i += 1;
        }

        smoothed_path
    }

    fn quadratic_bezier(&self, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
        let inv = 1.0 - t;
        (
            inv * inv * p0.0 + 2.0 * inv * t * p1.0 + t * t * p2.0,
            inv * inv * p0.1 + 2.0 * inv * t * p1.1 + t * t * p2.1,
        )
    }

    pub fn update(&mut self, traffic_signals: &TrafficSignals, tile_map: &Tilemap, delta_time: f32) {
        self.inventory.retain(|v| (tile_map.is_in_bounds(v.x, v.y) && tile_map.is_on_road(v.x, v.y)) && !v.destination_reached());
        for i in 0..self.inventory.len() {
            let signal_head = self.inventory[i].scan_for_signal_head(traffic_signals);
            let (x, y) = self.inventory[i].front();
            let current_tile = tile_map.get_tile(x, y).unwrap();

            match signal_head {
                Some(val) => {
                    if val.phase == LightPhase::Red && current_tile != Tile::RoadIntersect {
                        continue
                    } else {
                        self.inventory[i].update(delta_time);
                    }
                },
                None => {
                    if !self.is_vehicle_too_close(&self.inventory[i]) {
                        self.inventory[i].update(delta_time);
                    }
                }
            }
        }
    }

    pub fn is_vehicle_too_close(&self, vehicle: &Vehicle) -> bool {
        let direction = vehicle.direction();

        let distance_check = match direction {
            Direction::North => vehicle.y + -VEHICLE_FOLLOW_DISTANCE,
            Direction::South => vehicle.y + VEHICLE_FOLLOW_DISTANCE,
            Direction::East => vehicle.x + VEHICLE_FOLLOW_DISTANCE,
            Direction::West => vehicle.x + -VEHICLE_FOLLOW_DISTANCE,
            Direction::Null => 0.0
        };

        for v in &self.inventory {
            match direction {
                Direction::North | Direction::South => {
                    if (distance_check - 1.0..distance_check + 1.0).contains(&v.y) && (vehicle.x - 1.0..vehicle.x + 1.0).contains(&v.x) {
                        return true;
                    }
                },
                Direction::East | Direction::West => {
                    if (distance_check - 1.0..distance_check + 1.0).contains(&v.x) && (vehicle.y - 1.0..vehicle.y + 1.0).contains(&v.y) {
                        return true;
                    }
                },
                Direction::Null => return false,
            }
        }

        false
    }


    fn random_destination(&self, vehicle: &Vehicle) -> Option<(usize, usize)> {
        let vehicle_direction = vehicle.direction();
        let opposite = match vehicle_direction {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East  => Direction::West,
            Direction::West  => Direction::East,
            Direction::Null  => Direction::Null,
        };

        let height = self.pathfinding_map.len();
        let width = self.pathfinding_map[0].len();

        let mut edges: Vec<Direction> = [Direction::North, Direction::South, Direction::East, Direction::West]
            .into_iter()
            .filter(|d| *d != opposite)
            .collect();

        let mut rng = rng();
        edges.shuffle(&mut rng);

        for edge in edges {
            let point = match edge {
                Direction::North => (0..width).find(|&x| self.pathfinding_map[0][x]).map(|x| (x, 0)),
                Direction::South => (0..width).find(|&x| self.pathfinding_map[height - 1][x]).map(|x| (x, height - 1)),
                Direction::West  => (0..height).find(|&y| self.pathfinding_map[y][0]).map(|y| (0, y)),
                Direction::East  => (0..height).find(|&y| self.pathfinding_map[y][width - 1]).map(|y| (width - 1, y)),
                _ => None,
            };

            if point.is_some() {
                return point;
            }
        }

        None
    }
}

pub struct Vehicle {
    pub x: f32,
    pub y: f32,
    pub facing_angle: f64,
    pub speed: f32,
    pub color: Color,
    pub path: Option<Vec<Point>>,
    path_idx: usize
}

impl Vehicle {
    pub fn update(&mut self, delta_time: f32) {
        let Some(ref path) = self.path else { return; };
        if self.path_idx >= path.len() {
            return;
        }
        let target = path[self.path_idx];

        let dx = target.x as f32 - self.x;
        let dy = target.y as f32 - self.y;

        self.facing_angle = (dy as f64).atan2(dx as f64).to_degrees() + 90.0;

        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 5.0 {
            self.path_idx += 1;
            return;
        }

        self.x += self.speed * (dx / dist) * delta_time;
        self.y += self.speed * (dy / dist) * delta_time;

        if dist < 5.0 {
            self.path_idx += 1;
        }
    }

    pub fn destination_reached(&self) -> bool {
        let Some(ref path) = self.path else { return true; };
        self.path_idx >= path.len()
    }

    pub fn scan_for_signal_head(&self, traffic_signals: &TrafficSignals) -> Option<SignalHead> {
        let mut x_check = self.x;
        let mut y_check = self.y;

        let direction = self.direction();

        match direction {
            Direction::North => {
                y_check = self.y - (TILE_SIZE * 2.5);
            },
            Direction::South => {
                y_check = self.y + (TILE_SIZE * 2.4);
            },
            Direction::East => {
                x_check = self.x + (TILE_SIZE * 2.5);
            },
            Direction::West => {
                x_check = self.x - (TILE_SIZE * 2.4);
            },
            Direction::Null => {
                return None;
            }
        };

        traffic_signals.get_signal_head_by_position(x_check, y_check)
    }

    pub fn direction(&self) -> Direction {
        if self.speed == 0.0 {
            return Direction::Null;
        }

        let angle = self.facing_angle.rem_euclid(360.0);

        match angle {
            a if a >= 315.0 || a < 45.0  => Direction::North,
            a if a >= 135.0 && a < 225.0 => Direction::South,
            a if a >= 45.0  && a < 135.0 => Direction::East,
            _ => Direction::West,
        }
    }

    pub fn front(&self) -> (f32, f32) {
        let direction = self.direction();

        match direction {
            Direction::North => return (self.x + VEHICLE_WIDTH / 4.0, self.y),
            Direction::South => return (self.x - VEHICLE_WIDTH / 4.0, self.y),
            Direction::East => return (self.x, self.y + VEHICLE_HEIGHT / 4.0),
            Direction::West => return (self.x, self.y - VEHICLE_HEIGHT / 4.0),
            _ => return (0.0, 0.0)
        };
    }

    pub fn set_path(&mut self, path: Vec<Point>) {
        self.path = Some(path);
    }
}
