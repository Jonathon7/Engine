use std::collections::HashMap;
use crate::platform::tilemap::{Tile, Tilemap, TILE_SIZE, MAP_DATA, MAP_HEIGHT, MAP_WIDTH, IntoTileIndex};
use crate::traffic::{TrafficSignals, SignalHead, LightPhase};
use crate::pathfinding::{Point, astar};
use rand::{random_range, rng};
use rand::seq::SliceRandom;
use sdl3::pixels::Color;

const MAX_VEHICLE_LIMIT: usize = 150;
const VEHICLE_FOLLOW_DISTANCE: f32 = TILE_SIZE * 1.1;
pub const VEHICLE_WIDTH: f32 = 16.0;
pub const VEHICLE_HEIGHT: f32 = 24.0;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Null
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum RelativeDirection {
    Forward,
    Backward,
    Left,
    Right
}

#[derive(PartialEq, Debug)]
enum TurnType {
    Straight,
    Right,
    Left
}

pub struct Vehicles {
    pub inventory: Vec<Vehicle>,
    spawn_points: Vec<((usize, usize), Tile)>,
    pathfinding_map: Vec<Vec<bool>>,
    pub char_map: Vec<Vec<char>>,
    pub vehicles_by_position: HashMap<(usize, usize), usize>
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
            char_map,
            vehicles_by_position: HashMap::new()
        }
    }

    pub fn spawn_vehicle(&mut self, id: usize) {
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

            let neighbors = self.square_neighbors(x, y, 1);
            if neighbors.iter().any(|p| self.vehicles_by_position.get(p).is_some_and(|&v| v > 0) ) {
                println!("NO ROOM TO SPAWN VEHICLE");
                return;
            }

            let mut vehicle = Vehicle { id: id as u32, x: x as f32 * TILE_SIZE + x_pos_offset, y: y as f32 * TILE_SIZE + y_pos_offset, facing_angle: angle, speed: 70.0, color: Color::RGB(150, 150, 145), path: None, path_idx: 0 };
            let destination = self.random_destination(&vehicle).unwrap();
            let path = astar(&self.pathfinding_map, Point { x: x.try_into().unwrap(), y: y.try_into().unwrap() }, Point { x: destination.0.try_into().unwrap(), y: destination.1.try_into().unwrap() });
            vehicle.set_path(self.smooth_path(self.normalize_vehicle_path(&path.unwrap(), &vehicle), 12, vehicle.id));
            self.inventory.push(vehicle);
        }
    }

    fn square_neighbors(&self, x: usize, y: usize, radius: usize) -> Vec<(usize, usize)> {
        let mut points = Vec::new();
        let x_start = x.saturating_sub(radius);
        let y_start = y.saturating_sub(radius);
        let x_end = (x + radius).min(MAP_WIDTH - 1);
        let y_end = (y + radius).min(MAP_HEIGHT - 1);

        for nx in x_start..=x_end {
            for ny in y_start..=y_end {
                if nx != x || ny != y {
                    points.push((nx, ny));
                }
            }
        }

        points
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

    fn relative_direction_delta(&self, dir: Direction, relative_dir: RelativeDirection) -> (i32, i32) {
        match dir {
            Direction::North => {
                match relative_dir {
                    RelativeDirection::Forward => (-1, 0),
                    RelativeDirection::Backward => ( 1, 0),
                    RelativeDirection::Left => ( 0, -1),
                    RelativeDirection::Right => ( 0, 1)
                }
            },
            Direction::South => {
                match relative_dir {
                    RelativeDirection::Forward => (1, 0),
                    RelativeDirection::Backward => ( -1, 0),
                    RelativeDirection::Left => ( 0, 1),
                    RelativeDirection::Right => ( 0, -1)
                }
            },
            Direction::East => {
                match relative_dir {
                    RelativeDirection::Forward => (0, 1),
                    RelativeDirection::Backward => (0, -1),
                    RelativeDirection::Left => (-1, 0),
                    RelativeDirection::Right => (1, 0)
                }
            },
            Direction::West => {
                match relative_dir {
                    RelativeDirection::Forward => (0, -1),
                    RelativeDirection::Backward => (0, 1),
                    RelativeDirection::Left => (1, 0),
                    RelativeDirection::Right => (-1, 0)
                }
            },
            Direction::Null => {
                (0, 0)
            }
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
        let mut exit_idx = path.iter()
            .enumerate()
            .skip_while(|(_, p)| !self.is_intersection_tile(self.char_map[(p.y as usize)][(p.x as usize)]))
            .find(|(_, p)| !self.is_intersection_tile(self.char_map[(p.y as usize)][(p.x as usize)]))
            .map(|(i, _)| i)?;

        debug_assert!(exit_idx != 0);

        exit_idx += 1; 
        let exit = &path[exit_idx];
        let last_intersection_tile = &path[exit_idx - 1];

        let dx = exit.x - last_intersection_tile.x;
        let dy = exit.y - last_intersection_tile.y;

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
            return path.clone()
        }

        let mut normalized_path: Vec<Point> = Vec::new();
        let mut pos = path[0].clone();
        let mut pos_idx = 0;
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
                pos_idx += 1;
                normalized_path.push(pos.clone());
            } else {
                let exit_dir = self.find_exit_direction(&path[pos_idx + 1..]).unwrap();
                let turn_type = self.turn_type(&dir, &exit_dir);

                let mut zone_dir = dir.clone();
                let mut intersection_tile_count = 0;

                pos = next;
                pos_idx += 1;
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
                    pos_idx += 1;
                    normalized_path.push(pos.clone());
                }

                dir = exit_dir;
            }
        }

        normalized_path
    }
    
    fn smooth_path(&mut self, path: Vec<Point>, segments: usize, id: u32) -> Vec<Point> {
        let mut smoothed_path: Vec<Point> = Vec::new();
        let mut left_turn_encountered = false;

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

            if left_turn_encountered {
                left_turn_encountered = false;
                i += 1;
                continue;
            }

            let dx1 = (curr.x - prev.x) as f32;
            let dy1 = (curr.y - prev.y) as f32;
            let dx2 = (next.x - curr.x) as f32;
            let dy2 = (next.y - curr.y) as f32;

            let is_turn = (dx1.abs() > 0.01 && dy2.abs() > 0.01)
                       || (dy1.abs() > 0.01 && dx2.abs() > 0.01);

            if is_turn {
                let turn_cross = dx1 * dy2 - dy1 * dx2;

                if turn_cross < 0.0 {
                    // pop first intersection point
                    smoothed_path.pop();
                    left_turn_encountered = true;

                    let entry_dir = (self.dir_sign(dx1), self.dir_sign(dy1));
                    let exit_dir  = (self.dir_sign(dx2), self.dir_sign(dy2));

                    let lane_offset = TILE_SIZE * 0.35;

                    let entry_perp = (entry_dir.1, -entry_dir.0);
                    let exit_perp  = (exit_dir.1,  -exit_dir.0);

                    let p0 = (
                        prev.x as f32 - entry_dir.0 * lane_offset,
                        prev.y as f32 - entry_dir.1 * lane_offset
                    );
                    let p2 = (
                        next.x as f32 + exit_dir.0 * TILE_SIZE * 0.5,
                        next.y as f32 + exit_dir.1 * TILE_SIZE * 0.5
                    );

                    let p1 = (
                        curr.x as f32 + entry_perp.0 * lane_offset + exit_perp.0 * lane_offset,
                        curr.y as f32 + entry_perp.1 * lane_offset + exit_perp.1 * lane_offset,
                    );

                    let mid = ((p0.0 + p2.0) * 0.5, (p0.1 + p2.1) * 0.5);
                    let weight = 0.5;
                    let p1_weighted = (
                        mid.0 + (p1.0 - mid.0) * weight,
                        mid.1 + (p1.1 - mid.1) * weight,
                    );

                    for s in 0..=segments {
                        let t = s as f32 / segments as f32;
                        let (x, y) = self.quadratic_bezier(p0, p1_weighted, p2, t);
                        smoothed_path.push(Point { x: x as i32, y: y as i32 });
                    }
                } else {
                    let p0 = ((prev.x + curr.x) as f32 * 0.5, (prev.y + curr.y) as f32 * 0.5);
                    let p2 = ((curr.x + next.x) as f32 * 0.5, (curr.y + next.y) as f32 * 0.5);

                    const CORNER_ROUNDING: f32 = 0.05;
                    let seg_in = (((curr.x - prev.x).pow(2) + (curr.y - prev.y).pow(2)) as f32).sqrt();
                    let seg_out = (((next.x - curr.x).pow(2) + (next.y - curr.y).pow(2)) as f32).sqrt();
                    let offset = (TILE_SIZE * CORNER_ROUNDING).min(seg_in.min(seg_out) * 0.05);

                    let curr_offset = (
                        curr.x as f32 + (dx2.signum() + dx1.signum()) * offset,
                        curr.y as f32 + (dy2.signum() + dy1.signum()) * offset,
                    );
                    curr.x = curr_offset.0 as i32;
                    curr.y = curr_offset.1 as i32;

                    for s in 0..=segments {
                        let t = s as f32 / segments as f32;
                        let (x, y) = self.quadratic_bezier(p0, (curr.x as f32, curr.y as f32), p2, t);
                        smoothed_path.push(Point { x: x as i32, y: y as i32 });
                    }
                }
            } else {
                smoothed_path.push(curr);
            }

            i += 1;
        }

        smoothed_path
    }

    fn dir_sign(&self, v: f32) -> f32 {
        if v > 0.01 { 1.0 } else if v < -0.01 { -1.0 } else { 0.0 }
    }

    fn quadratic_bezier(&self, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
        let inv = 1.0 - t;
        (
            inv * inv * p0.0 + 2.0 * inv * t * p1.0 + t * t * p2.0,
            inv * inv * p0.1 + 2.0 * inv * t * p1.1 + t * t * p2.1,
        )
    }

    pub fn update(&mut self, traffic_signals: &TrafficSignals, tile_map: &Tilemap, delta_time: f32) {
        let mut vehicles_to_remove: Vec<usize> = Vec::new();
        for (i, v) in self.inventory.iter().enumerate() {
            if !tile_map.is_in_bounds(v.x, v.y) || v.destination_reached() {
                vehicles_to_remove.push(i);
            }
        }

        self.swap_remove_many(vehicles_to_remove);

        for i in 0..self.inventory.len() {
            let signal_head = self.inventory[i].scan_for_signal_head(traffic_signals, RelativeDirection::Forward);
            let signal_head_left = self.inventory[i].scan_for_signal_head(traffic_signals, RelativeDirection::Left);
            let (x, y) = self.inventory[i].front();
            let current_tile = tile_map.get_tile(x, y).unwrap();

            let old_pos_x = self.inventory[i].x;
            let old_pos_y = self.inventory[i].y;

            // ****** DEBUG ******
            let next_turn = self.next_turn(&self.inventory[i]);
            match next_turn {
                TurnType::Left => {
                    self.inventory[i].set_color(Color::CYAN);
                },
                _ => {
                    self.inventory[i].set_color(Color::RED);
                }
            }
            // *******************

            match signal_head {
                Some(val) => {
                    if val.phase == LightPhase::Red && current_tile != Tile::RoadIntersect {
                        continue
                    }

                    if (val.phase == LightPhase::Green || val.phase == LightPhase::Yellow) && signal_head_left.is_some() {//&& current_tile != Tile::RoadIntersect {
                        let turn_type = self.next_turn(&self.inventory[i]);
                        if turn_type == TurnType::Left && (signal_head_left.unwrap().phase == LightPhase::Green || signal_head.unwrap().phase == LightPhase::Yellow) {
                            let mut can_go = true;
                            let oncoming_vehicles = self.oncoming_vehicles(&self.inventory[i]);

                            let first_vehicle = oncoming_vehicles.iter().find(|v| v.is_some());

                            if first_vehicle.is_some() && self.next_turn(&self.inventory[first_vehicle.unwrap().unwrap()]) == TurnType::Left {
                                can_go = true;
                            } else if oncoming_vehicles.iter().all(|v| v.is_none()) {
                                can_go = true;
                            } else {
                                can_go = false;
                            }

                            if !can_go {
                                continue;
                            }
                        }
                        self.inventory[i].update(delta_time);
                    }
                    else {
                        self.inventory[i].update(delta_time);
                    }
                },
                None => {
                    if !self.is_vehicle_too_close(&self.inventory[i]) {
                        self.inventory[i].update(delta_time);
                    }
                }
            }

            let old_key = (old_pos_x.into_index().unwrap(), old_pos_y.into_index().unwrap());
            let new_key = (self.inventory[i].x.into_index().unwrap(), self.inventory[i].y.into_index().unwrap());
            if old_key != new_key {
                self.vehicles_by_position.remove(&old_key);
                self.vehicles_by_position.insert(new_key, i);
            }
        }
    }

    fn swap_remove_many(&mut self, mut to_remove: Vec<usize>) {
        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        to_remove.dedup();

        for &index in &to_remove {
            let last = self.inventory.len() - 1;
            self.inventory.swap_remove(index);

            self.vehicles_by_position.retain(|_, value| *value != index);

            if index != last {
                for value in self.vehicles_by_position.values_mut() {
                    if *value == last {
                        *value = index;
                    }
                }
            }
        }
    }

    fn next_turn(&self, vehicle: &Vehicle) -> TurnType {
        let mut prev_point = (0, 0);
        let path: Vec<Point> = vehicle.path.clone().unwrap()[vehicle.path_idx..].iter().filter_map(|p| {
            let y = p.y.into_index().unwrap() as i32;
            let x = p.x.into_index().unwrap() as i32;

            if y == prev_point.0 && x == prev_point.1 {
                prev_point = (y, x);
                None
            } else {
                prev_point = (y, x);
                Some(Point { x, y })
            }
        }).collect();

        let Some(exit_dir) = self.find_exit_direction(&path) else { return TurnType::Straight };

        self.turn_type(&vehicle.direction(), &exit_dir)
    }

    pub fn is_vehicle_too_close(&self, vehicle: &Vehicle) -> bool {
        let direction = vehicle.direction();

        if direction == Direction::Null {
            return false;
        }

        for v in &self.inventory {
            if std::ptr::eq(v, vehicle) {
                continue;
            }

            let too_close = match direction {
                Direction::North => {
                    (vehicle.y - VEHICLE_FOLLOW_DISTANCE..vehicle.y).contains(&v.y)
                        && (vehicle.x - 8.0..vehicle.x + 8.0).contains(&v.x)
                }
                Direction::South => {
                    (vehicle.y..vehicle.y + VEHICLE_FOLLOW_DISTANCE).contains(&v.y)
                        && (vehicle.x - 8.0..vehicle.x + 8.0).contains(&v.x)
                }
                Direction::East => {
                    (vehicle.x..vehicle.x + VEHICLE_FOLLOW_DISTANCE).contains(&v.x)
                        && (vehicle.y - 8.0..vehicle.y + 8.0).contains(&v.y)
                }
                Direction::West => {
                    (vehicle.x - VEHICLE_FOLLOW_DISTANCE..vehicle.x).contains(&v.x)
                        && (vehicle.y - 8.0..vehicle.y + 8.0).contains(&v.y)
                }
                Direction::Null => unreachable!(),
            };

            if too_close {
                return true;
            }
        }

        false
    }

    fn oncoming_vehicles(&self, vehicle: &Vehicle) -> Vec<Option<usize>> {
        let max_tries = 4;
        let mut tries = 0;
        let (x_dir, y_dir) = self.direction_delta(vehicle.direction());
        let mut y = vehicle.y;
        let mut x = vehicle.x;

        loop {
            tries += 1;
            if tries >= max_tries {
                println!("max_tries");
                return Vec::new();
            }

            y += y_dir as f32 * TILE_SIZE;
            x += x_dir as f32 * TILE_SIZE;

            let y_index = y.into_index().unwrap();
            let x_index = x.into_index().unwrap();

            if y_index as usize > MAP_HEIGHT || x_index as usize > MAP_WIDTH {
                return Vec::new();
            }

            if self.is_intersection_tile(self.char_map[y_index as usize][x_index as usize]) {
                continue;
            }

            let (left_dy, left_dx) = self.relative_direction_delta(vehicle.direction(), RelativeDirection::Left);
            let left_y = (left_dy + y_index as i32) as usize;
            let left_x = (left_dx + x_index as i32) as usize;
            
            let tiles_to_check = [
                                  //((left_x as i32 + x_dir * -1) as usize, (left_y as i32 + y_dir * -1) as usize), // -1,
                                  (left_x, left_y), // light tile
                                  ((left_x as i32 + x_dir * 1) as usize, (left_y as i32 + y_dir * 1) as usize), // + 1,
                                  ((left_x as i32 + x_dir * 2) as usize, (left_y as i32 + y_dir * 2) as usize), // + 2
                                  ((left_x as i32 + x_dir * 3) as usize, (left_y as i32 + y_dir * 3) as usize), // + 3
                                 ];

            return tiles_to_check.iter().map(|t| {
                self.vehicles_by_position.get(t).copied()
            }).collect();
        }
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

            // let left_only = match vehicle_direction {
            //     Direction::North => Direction::West,
            //     Direction::South => Direction::East,
            //     Direction::East => Direction::North,
            //     Direction::West => Direction::South,
            //     Direction::Null => Direction::Null
            // };

            let height = self.pathfinding_map.len();
            let width = self.pathfinding_map[0].len();

            let mut edges: Vec<Direction> = vec![Direction::North, Direction::South, Direction::East, Direction::West]
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
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub facing_angle: f64,
    pub speed: f32,
    pub color: Color,
    pub path: Option<Vec<Point>>,
    pub path_idx: usize
}

impl PartialEq for Vehicle {
    fn eq(&self, other: &Self) -> bool {
        self.x.into_index().unwrap() == other.x.into_index().unwrap() &&
        self.y.into_index().unwrap() == other.y.into_index().unwrap()
    }
}

impl Vehicle {
    // DEBUG
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

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
        if dist < 7.0 {
            self.path_idx += 1;
            return;
        }

        self.x += self.speed * (dx / dist) * delta_time;
        self.y += self.speed * (dy / dist) * delta_time;

        if dist < 7.0 {
            self.path_idx += 1;
        }
    }

    pub fn destination_reached(&self) -> bool {
        let Some(ref path) = self.path else { return true; };
        self.path_idx >= path.len()
    }

    pub fn scan_for_signal_head(&self, traffic_signals: &TrafficSignals, scan_dir: RelativeDirection) -> Option<SignalHead> {
        const SCAN_NUM_TILES: u8 = 3;

        let vehicle_direction = self.direction();

        let (y_dir, x_dir): (i32, i32) = match vehicle_direction {
            Direction::North => {
                match scan_dir {
                    RelativeDirection::Forward => (-1, 0),
                    RelativeDirection::Backward => ( 1, 0),
                    RelativeDirection::Left => ( 0, -1),
                    RelativeDirection::Right => ( 0, 1)
                }
            },
            Direction::South => {
                match scan_dir {
                    RelativeDirection::Forward => (1, 0),
                    RelativeDirection::Backward => ( -1, 0),
                    RelativeDirection::Left => ( 0, 1),
                    RelativeDirection::Right => ( 0, -1)
                }
            },
            Direction::East => {
                match scan_dir {
                    RelativeDirection::Forward => (0, 1),
                    RelativeDirection::Backward => (0, -1),
                    RelativeDirection::Left => (-1, 0),
                    RelativeDirection::Right => (1, 0)
                }
            },
            Direction::West => {
                match scan_dir {
                    RelativeDirection::Forward => (0, -1),
                    RelativeDirection::Backward => (0, 1),
                    RelativeDirection::Left => (1, 0),
                    RelativeDirection::Right => (-1, 0)
                }
            },
            Direction::Null => {
                (0, 0)
            }
        };

        for i in 1..=SCAN_NUM_TILES {
            let Some(signal_head) = traffic_signals.get_signal_head_by_position(self.x + (x_dir as f32 * i as f32 * TILE_SIZE), self.y + (y_dir as f32 * i as f32 * TILE_SIZE)) else { continue; };
            return Some(signal_head);
        }

        None
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
