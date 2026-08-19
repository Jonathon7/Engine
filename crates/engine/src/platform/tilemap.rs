use sdl3::render::{WindowCanvas, FRect, Texture};
use sdl3::pixels::Color;
use crate::platform::camera::{Camera};
use crate::vehicle::{Vehicle, VEHICLE_WIDTH, VEHICLE_HEIGHT};
use crate::traffic::{SignalHead, LightPhase};

const DEBUG_TILE_BORDERS: bool = true;

// pub const MAP_DATA: &str = "\
// .............v^.............
// ....BB.......v^.......B.....
// ....BB.......v^.......BB....
// .............v^.............
// .............v^.............
// -------------vL-------------
// <<<<<<<<<<<<L++<<<<<<<<<<<<]
// [>>>>>>>>>>>>++L>>>>>>>>>>>>
// -------------L^-------------
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// .............v^.............
// ";

pub const MAP_DATA: &str = "\
.............{^.............
....BB.......v^.......B.....
....BB.......v^.......BB....
.............v^.............
.............v^.............
-------------vL-------------
<<<<<<<<<<<<L++<<<<<<<<<<<<]
[>>>>>>>>>>>>++L>>>>>>>>>>>>
-------------L^-------------
.............v^.............
.............v^.............
.............v^.............
.............v^.............
.............v^.............
.............v^.............
.............v^.............
.............vL.............
<<<<<<<<<<<<L++<<<<<<<<<<<<]
[>>>>>>>>>>>>++L>>>>>>>>>>>>
.............L^.............
.............v^.............
.............v^.............
.............v^.............
.............v^.............
.............v^.............
.............v^.............
.............v^.............
.............v}.............
";

pub const TILE_SIZE: f32 = 96.0;
pub const MAP_WIDTH: usize = 28;
pub const MAP_HEIGHT: usize = 28;
pub const WORLD_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;
pub const WORLD_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_SIZE;

pub const SCREEN_WIDTH: u32 = 800;
pub const SCREEN_HEIGHT: u32 = 608;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tile {
    Grass,
    RoadH,
    RoadV,
    RoadVNSpawn,
    RoadVSSpawn,
    RoadHESpawn,
    RoadHWSpawn,
    RoadIntersect,
    Stoplight,
    Sidewalk,
    Building
}

pub trait IntoTileIndex {
    fn into_index(self) -> Option<usize>;
}

impl IntoTileIndex for usize {
    fn into_index(self) -> Option<usize> {
        Some(self)
    }
}

impl IntoTileIndex for i32 {
    fn into_index(self) -> Option<usize> {
        if self < 0 {
            None
        } else {
            let index = (self as f32 / TILE_SIZE).floor() as usize;
            if index > MAP_WIDTH {
                None
            } else {
                Some(index)
            }
        }
    }
}

impl IntoTileIndex for f32 {
    fn into_index(self) -> Option<usize> {
         if self.is_nan() {
             None
         } else if self <= 0.0 {
             None
         } else {
             let index = (self / TILE_SIZE).floor() as usize;

             if index > MAP_WIDTH {
                 None
             } else {
                 Some(index)
             }
         }
    }
}

pub struct Tilemap {
    pub map: Vec<Vec<Tile>>
}

impl Tilemap {
    pub fn new() -> Self {
        Tilemap {
            map: build_map()
        }
    }

    pub fn get_tile<T: IntoTileIndex>(&self, x: T, y: T) -> Option<Tile> {
        let Some(index_x) = x.into_index() else { return None };
        let Some(index_y) = y.into_index() else { return None };

        Some(self.map[index_y][index_x])
    }

    pub fn is_on_road<T: IntoTileIndex>(&self, x: T, y: T) -> bool {
        let Some(tile) = self.get_tile(x, y) else { return false; };

        use Tile::*;
        match tile {
            RoadH | RoadV | RoadVNSpawn | RoadVSSpawn | RoadHESpawn | RoadHWSpawn | RoadIntersect | Stoplight => {
                return true;
            },
            _ => { return false; }
        };
    }

    pub fn is_in_bounds<T: IntoTileIndex>(&self, x: T, y: T) -> bool {
        let Some(index_x) = x.into_index() else { return false; };
        let Some(index_y) = y.into_index() else { return false; };

        if index_x >= MAP_WIDTH || index_y >= MAP_HEIGHT {
            return false;
        }

        true
    }
}

pub fn build_map() -> Vec<Vec<Tile>> {
    MAP_DATA.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.chars().map(|c| match c {
                '.' => Tile::Grass,
                'B' => Tile::Building,
                '-' => Tile::Sidewalk,
                '>' | '<' => Tile::RoadH,
                '^' | 'v' => Tile::RoadV,
                '}' => Tile::RoadVNSpawn,
                '{' => Tile::RoadVSSpawn,
                '[' => Tile::RoadHESpawn,
                ']' => Tile::RoadHWSpawn,
                '+' => Tile::RoadIntersect,
                'L' => Tile::Stoplight,
                _ => Tile::Grass
            }).collect()
        }).collect()
}

fn draw_tile(canvas: &mut WindowCanvas, tile: Tile, screen_x: f32, screen_y: f32) {
    let rect = FRect::new(screen_x, screen_y, TILE_SIZE, TILE_SIZE);

    match tile {
        Tile::Grass => {
            canvas.set_draw_color(Color::RGB(80, 140, 60));
            canvas.fill_rect(rect).unwrap();
        }
        Tile::RoadH | Tile::RoadHESpawn | Tile::RoadHWSpawn => {
            canvas.set_draw_color(Color::RGB(60, 60, 65));
            canvas.fill_rect(rect).unwrap();
            canvas.set_draw_color(Color::RGB(230, 220, 100));
           // canvas.fill_rect(FRect::new(screen_x + 4.0, screen_y + 14.0, 8.0, 4.0)).unwrap();
            // canvas.fill_rect(FRect::new(screen_x + 20.0, screen_y + 14.0, 8.0, 4.0)).unwrap();
        }
        Tile::RoadV | Tile::RoadVNSpawn | Tile::RoadVSSpawn => {
            canvas.set_draw_color(Color::RGB(60, 60, 65));
            canvas.fill_rect(rect).unwrap();
            canvas.set_draw_color(Color::RGB(230, 220, 100));
            // canvas.fill_rect(FRect::new(screen_x + 14.0, screen_y + 4.0, 4.0, 8.0)).unwrap();
            // canvas.fill_rect(FRect::new(screen_x + 14.0, screen_y + 20.0, 4.0, 8.0)).unwrap();
        }
        Tile::RoadIntersect => {
            canvas.set_draw_color(Color::RGB(60, 60, 65));
            canvas.fill_rect(rect).unwrap();
        }
        Tile::Stoplight => {
        }
        Tile::Sidewalk => {
            canvas.set_draw_color(Color::RGB(180, 180, 175));
            canvas.fill_rect(rect).unwrap();
            canvas.set_draw_color(Color::RGB(150, 150, 145));
            canvas.fill_rect(FRect::new(screen_x, screen_y, TILE_SIZE, 1.0)).unwrap();
            canvas.fill_rect(FRect::new(screen_x, screen_y + 16.0, TILE_SIZE, 1.0)).unwrap();
        }
        Tile::Building => {
            canvas.set_draw_color(Color::RGB(120, 100, 90));
            canvas.fill_rect(rect).unwrap();
            canvas.set_draw_color(Color::RGB(200, 220, 240));
            canvas.fill_rect(FRect::new(screen_x + 6.0, screen_y + 6.0, 8.0, 8.0)).unwrap();
            canvas.fill_rect(FRect::new(screen_x + 18.0, screen_y + 6.0, 8.0, 8.0)).unwrap();
            canvas.fill_rect(FRect::new(screen_x + 6.0, screen_y + 18.0, 8.0, 8.0)).unwrap();
            canvas.fill_rect(FRect::new(screen_x + 18.0, screen_y + 18.0, 8.0, 8.0)).unwrap();
        }
    }

    if DEBUG_TILE_BORDERS {
        let rect = FRect::new(screen_x, screen_y, TILE_SIZE, TILE_SIZE);
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 80));
        canvas.draw_rect(rect).unwrap();
    }
}

fn fill_circle(canvas: &mut WindowCanvas, circle_x: f32, circle_y: f32, radius: f32) {
    let r = radius as i32;
    for dy in -r..=r {
        let dx = ((r * r - dy * dy) as f32).sqrt();
        let strip = FRect::new(
            circle_x - dx,
            circle_y + dy as f32,
            dx * 2.0,
            1.0,
        );
        canvas.fill_rect(strip).unwrap();
    }
}

pub fn draw_visible_tiles(canvas: &mut WindowCanvas, map: &[Vec<Tile>], camera: &Camera) {
    let x0 = (camera.x / TILE_SIZE).floor() as i32;
    let y0 = (camera.y / TILE_SIZE).floor() as i32;
    let x1 = ((camera.x + SCREEN_WIDTH as f32) / TILE_SIZE).ceil() as i32 + 1;
    let y1 = ((camera.y + SCREEN_HEIGHT as f32) / TILE_SIZE).ceil() as i32 + 1;

    let x0 = x0.max(0) as usize;
    let y0 = y0.max(0) as usize;
    let x1 = (x1 as usize).min(MAP_WIDTH);
    let y1 = (y1 as usize).min(MAP_HEIGHT);

    for y in y0..y1 {
        for x in x0..x1 {
            let world_x = x as f32 * TILE_SIZE;
            let world_y = y as f32 * TILE_SIZE;
            let (screen_x, screen_y) = camera.world_to_screen(world_x, world_y);
            draw_tile(canvas, map[y][x], screen_x, screen_y);
        }
    }
}

pub fn draw_vehicle(canvas: &mut WindowCanvas, vehicle: &Vehicle, texture: &Texture, camera: &Camera) {
    let (screen_x, screen_y) = camera.world_to_screen(vehicle.x, vehicle.y);
 
    let (w, h) = (VEHICLE_WIDTH, VEHICLE_HEIGHT);
    let radius = 24.0;

    if screen_x + radius < 0.0 || screen_x - radius > SCREEN_WIDTH as f32
        || screen_y + radius < 0.0 || screen_y - radius > SCREEN_HEIGHT as f32
    {
        return;
    }
 
    let distance = FRect::new(screen_x - w / 2.0, screen_y - h / 2.0, w, h);

    canvas.copy_ex(
        texture,
        None,           // src rect (full texture)
        Some(distance),      // dest rect
        vehicle.facing_angle, // rotation in degrees
        None,           // center of rotation (None = center of dst)
        false, false,   // flip horizontal, vertical
    ).unwrap();
}

pub fn draw_traffic_signals(canvas: &mut WindowCanvas, traffic_signals: &Vec<(String, [SignalHead; 4])>, camera: &Camera) {
    for traffic_signal in traffic_signals {
        let signal_group = &traffic_signal.1;
        for signal_head in signal_group {
            let phase = &signal_head.phase;
            let (screen_x, screen_y) = camera.world_to_screen(signal_head.x as f32 * TILE_SIZE, signal_head.y as f32 * TILE_SIZE);

            canvas.set_draw_color(Color::RGB(60, 60, 65));
            canvas.fill_rect(FRect::new(screen_x, screen_y, TILE_SIZE, TILE_SIZE)).unwrap();

            canvas.set_draw_color(Color::RGB(30, 30, 35));
            canvas.fill_rect(FRect::new(screen_x + 10.0, screen_y + 4.0, 12.0, 24.0)).unwrap();

            let bright_red    = Color::RGB(230,  40,  40);
            let bright_yellow = Color::RGB(240, 210,  60);
            let bright_green  = Color::RGB( 60, 220,  80);
            let dim_red       = Color::RGB( 70,  20,  20);
            let dim_yellow    = Color::RGB( 70,  60,  20);
            let dim_green     = Color::RGB( 20,  60,  25);

            let (red_color, yellow_color, green_color) = match phase {
                LightPhase::Red    => (bright_red, dim_yellow, dim_green),
                LightPhase::Yellow => (dim_red, bright_yellow, dim_green),
                LightPhase::Green  => (dim_red, dim_yellow, bright_green),
            };

            canvas.set_draw_color(red_color);
            fill_circle(canvas, screen_x + 16.0, screen_y +  9.0, 3.0);
            canvas.set_draw_color(yellow_color);
            fill_circle(canvas, screen_x + 16.0, screen_y + 16.0, 3.0);
            canvas.set_draw_color(green_color);
            fill_circle(canvas, screen_x + 16.0, screen_y + 23.0, 3.0);
        }

    }
}
