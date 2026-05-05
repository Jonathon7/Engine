use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Scancode};
use sdl3::pixels::Color;
use sdl3::render::FRect;
use sdl3::render::WindowCanvas;
use std::time::{Duration, Instant};

// --- World and screen --------------------------------------------------------

const TILE_SIZE: f32 = 32.0;

// The world is bigger than the screen now. The screen is a viewport into it.
const MAP_WIDTH: usize = 60;
const MAP_HEIGHT: usize = 45;
const WORLD_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;   // 1920
const WORLD_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_SIZE; // 1440

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 608;

const CAMERA_SPEED: f32 = 400.0; // pixels/second

// --- Tile types --------------------------------------------------------------

#[derive(Copy, Clone, PartialEq)]
enum Tile {
    Grass,
    RoadH,
    RoadV,
    RoadCross,
    Sidewalk,
    Building,
}

// --- Camera ------------------------------------------------------------------
//
// The camera is a position in WORLD space pointing at the top-left corner of
// what's currently visible. To get from world coords to screen coords:
//   screen = world - camera
// To get from screen coords back to world (e.g. for mouse clicks):
//   world = screen + camera

struct Camera {
    x: f32,
    y: f32,
}

impl Camera {
    fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        (wx - self.x, wy - self.y)
    }

    /// Clamp the camera so it can't show area outside the world bounds.
    /// If the world is smaller than the screen on some axis, pin to 0.
    fn clamp_to_world(&mut self) {
        let max_x = (WORLD_WIDTH - SCREEN_WIDTH as f32).max(0.0);
        let max_y = (WORLD_HEIGHT - SCREEN_HEIGHT as f32).max(0.0);
        self.x = self.x.clamp(0.0, max_x);
        self.y = self.y.clamp(0.0, max_y);
    }
}

// --- Vehicle entity ----------------------------------------------------------

struct Vehicle {
    x: f32, // WORLD coordinates — vehicles don't know the camera exists
    y: f32,
    vx: f32,
    vy: f32,
    color: Color,
}

impl Vehicle {
    fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Wrap around the WORLD, not the screen
        if self.x < -32.0 {
            self.x = WORLD_WIDTH;
        }
        if self.x > WORLD_WIDTH {
            self.x = -32.0;
        }
        if self.y < -32.0 {
            self.y = WORLD_HEIGHT;
        }
        if self.y > WORLD_HEIGHT {
            self.y = -32.0;
        }
    }

    fn is_vertical(&self) -> bool {
        self.vy.abs() > self.vx.abs()
    }
}

// --- Map construction --------------------------------------------------------

fn build_map() -> Vec<Vec<Tile>> {
    let mut map = vec![vec![Tile::Grass; MAP_WIDTH]; MAP_HEIGHT];

    // A grid of horizontal roads
    let h_road_rows = [10usize, 25, 40];
    for &row in &h_road_rows {
        for x in 0..MAP_WIDTH {
            map[row - 1][x] = Tile::Sidewalk;
            map[row][x] = Tile::RoadH;
            map[row + 1][x] = Tile::Sidewalk;
        }
    }

    // Vertical roads
    let v_road_cols = [15usize, 35, 55];
    for &col in &v_road_cols {
        for y in 0..MAP_HEIGHT {
            map[y][col - 1] = Tile::Sidewalk;
            map[y][col] = Tile::RoadV;
            map[y][col + 1] = Tile::Sidewalk;
        }
    }

    // Fix up intersections
    for &row in &h_road_rows {
        for &col in &v_road_cols {
            map[row][col] = Tile::RoadCross;
            map[row - 1][col] = Tile::RoadV;
            map[row + 1][col] = Tile::RoadV;
            map[row][col - 1] = Tile::RoadH;
            map[row][col + 1] = Tile::RoadH;
        }
    }

    // Scatter some 2x2 buildings on grass blocks
    let buildings = [
        (3, 3), (7, 3), (20, 3), (25, 3), (40, 3), (45, 3),
        (3, 17), (7, 17), (20, 17), (25, 17), (40, 17), (45, 17),
        (3, 32), (7, 32), (20, 32), (25, 32), (40, 32), (45, 32),
    ];
    for (bx, by) in buildings {
        for dx in 0..2 {
            for dy in 0..2 {
                if by + dy < MAP_HEIGHT && bx + dx < MAP_WIDTH
                    && map[by + dy][bx + dx] == Tile::Grass
                {
                    map[by + dy][bx + dx] = Tile::Building;
                }
            }
        }
    }

    map
}

// --- Rendering ---------------------------------------------------------------

fn draw_tile(canvas: &mut WindowCanvas, tile: Tile, sx: f32, sy: f32) {
    // sx/sy are SCREEN coordinates — the caller has already converted from world.
    let rect = FRect::new(sx, sy, TILE_SIZE, TILE_SIZE);

    match tile {
        Tile::Grass => {
            canvas.set_draw_color(Color::RGB(80, 140, 60));
            canvas.fill_rect(rect).unwrap();
        }
        Tile::RoadH => {
            canvas.set_draw_color(Color::RGB(60, 60, 65));
            canvas.fill_rect(rect).unwrap();
            canvas.set_draw_color(Color::RGB(230, 220, 100));
            canvas.fill_rect(FRect::new(sx + 4.0, sy + 14.0, 8.0, 4.0)).unwrap();
            canvas.fill_rect(FRect::new(sx + 20.0, sy + 14.0, 8.0, 4.0)).unwrap();
        }
        Tile::RoadV => {
            canvas.set_draw_color(Color::RGB(60, 60, 65));
            canvas.fill_rect(rect).unwrap();
            canvas.set_draw_color(Color::RGB(230, 220, 100));
            canvas.fill_rect(FRect::new(sx + 14.0, sy + 4.0, 4.0, 8.0)).unwrap();
            canvas.fill_rect(FRect::new(sx + 14.0, sy + 20.0, 4.0, 8.0)).unwrap();
        }
        Tile::RoadCross => {
            canvas.set_draw_color(Color::RGB(60, 60, 65));
            canvas.fill_rect(rect).unwrap();
        }
        Tile::Sidewalk => {
            canvas.set_draw_color(Color::RGB(180, 180, 175));
            canvas.fill_rect(rect).unwrap();
            canvas.set_draw_color(Color::RGB(150, 150, 145));
            canvas.fill_rect(FRect::new(sx, sy, TILE_SIZE, 1.0)).unwrap();
            canvas.fill_rect(FRect::new(sx, sy + 16.0, TILE_SIZE, 1.0)).unwrap();
        }
        Tile::Building => {
            canvas.set_draw_color(Color::RGB(120, 100, 90));
            canvas.fill_rect(rect).unwrap();
            canvas.set_draw_color(Color::RGB(200, 220, 240));
            canvas.fill_rect(FRect::new(sx + 6.0, sy + 6.0, 8.0, 8.0)).unwrap();
            canvas.fill_rect(FRect::new(sx + 18.0, sy + 6.0, 8.0, 8.0)).unwrap();
            canvas.fill_rect(FRect::new(sx + 6.0, sy + 18.0, 8.0, 8.0)).unwrap();
            canvas.fill_rect(FRect::new(sx + 18.0, sy + 18.0, 8.0, 8.0)).unwrap();
        }
    }
}

fn draw_vehicle(canvas: &mut WindowCanvas, v: &Vehicle, camera: &Camera) {
    let (sx, sy) = camera.world_to_screen(v.x, v.y);

    // Skip vehicles outside the screen — saves a few draws.
    let (w, h) = if v.is_vertical() { (16.0, 24.0) } else { (24.0, 14.0) };
    if sx + w < 0.0 || sx > SCREEN_WIDTH as f32 || sy + h < 0.0 || sy > SCREEN_HEIGHT as f32 {
        return;
    }

    canvas.set_draw_color(v.color);
    canvas.fill_rect(FRect::new(sx, sy, w, h)).unwrap();

    canvas.set_draw_color(Color::RGB(40, 60, 80));
    let windshield = if v.is_vertical() {
        if v.vy > 0.0 {
            FRect::new(sx + 2.0, sy + 14.0, 12.0, 6.0)
        } else {
            FRect::new(sx + 2.0, sy + 4.0, 12.0, 6.0)
        }
    } else if v.vx > 0.0 {
        FRect::new(sx + 14.0, sy + 2.0, 6.0, 10.0)
    } else {
        FRect::new(sx + 4.0, sy + 2.0, 6.0, 10.0)
    };
    canvas.fill_rect(windshield).unwrap();
}

/// Draw only the tiles that are actually visible given the camera position.
/// This is the key optimization once your world is bigger than the screen.
fn draw_visible_tiles(canvas: &mut WindowCanvas, map: &[Vec<Tile>], camera: &Camera) {
    // Figure out the range of map indices that overlap the screen.
    // We extend by 1 tile on each side to handle partial tiles at the edges.
    let x0 = (camera.x / TILE_SIZE).floor() as i32;
    let y0 = (camera.y / TILE_SIZE).floor() as i32;
    let x1 = ((camera.x + SCREEN_WIDTH as f32) / TILE_SIZE).ceil() as i32 + 1;
    let y1 = ((camera.y + SCREEN_HEIGHT as f32) / TILE_SIZE).ceil() as i32 + 1;

    // Clamp to the actual map bounds so we don't index out-of-range.
    let x0 = x0.max(0) as usize;
    let y0 = y0.max(0) as usize;
    let x1 = (x1 as usize).min(MAP_WIDTH);
    let y1 = (y1 as usize).min(MAP_HEIGHT);

    for y in y0..y1 {
        for x in x0..x1 {
            let world_x = x as f32 * TILE_SIZE;
            let world_y = y as f32 * TILE_SIZE;
            let (sx, sy) = camera.world_to_screen(world_x, world_y);
            draw_tile(canvas, map[y][x], sx, sy);
        }
    }
}

// --- Main loop ---------------------------------------------------------------

fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Tilemap with Camera (WASD/arrows to move)", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let map = build_map();
    let mut camera = Camera { x: 0.0, y: 0.0 };

    // Vehicles in WORLD coordinates, placed on the various roads.
    let mut vehicles = vec![
        Vehicle { x: 0.0,         y: 10.0 * TILE_SIZE + 4.0,  vx:  80.0, vy:   0.0, color: Color::RGB(220,  50,  50) },
        Vehicle { x: WORLD_WIDTH, y: 10.0 * TILE_SIZE + 16.0, vx: -60.0, vy:   0.0, color: Color::RGB( 50, 100, 220) },
        Vehicle { x: 200.0,       y: 25.0 * TILE_SIZE + 4.0,  vx: 100.0, vy:   0.0, color: Color::RGB(220, 130,  60) },
        Vehicle { x: WORLD_WIDTH, y: 25.0 * TILE_SIZE + 16.0, vx: -75.0, vy:   0.0, color: Color::RGB(180,  80, 180) },
        Vehicle { x: 0.0,         y: 40.0 * TILE_SIZE + 4.0,  vx:  90.0, vy:   0.0, color: Color::RGB(120, 220, 220) },
        Vehicle { x: 15.0 * TILE_SIZE + 4.0,  y: 0.0,           vx: 0.0, vy:  70.0, color: Color::RGB(240, 200,  40) },
        Vehicle { x: 15.0 * TILE_SIZE + 16.0, y: WORLD_HEIGHT,  vx: 0.0, vy: -90.0, color: Color::RGB( 60, 180,  60) },
        Vehicle { x: 35.0 * TILE_SIZE + 4.0,  y: 100.0,         vx: 0.0, vy:  85.0, color: Color::RGB(220, 220, 240) },
        Vehicle { x: 55.0 * TILE_SIZE + 16.0, y: WORLD_HEIGHT,  vx: 0.0, vy: -65.0, color: Color::RGB(140, 140, 140) },
    ];

    let mut last_frame = Instant::now();

    'running: loop {
        // ----- input (events for one-shot actions like quitting) -----
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        // ----- input (held keys for continuous movement) -----
        // Polling KeyboardState every frame gives smooth motion.
        // KeyDown/KeyUp events would only fire once per press — wrong for held movement.
        let kb = event_pump.keyboard_state();
        let mut dx: f32 = 0.0;
        let mut dy: f32 = 0.0;
        if kb.is_scancode_pressed(Scancode::W) || kb.is_scancode_pressed(Scancode::Up) {
            dy -= 1.0;
        }
        if kb.is_scancode_pressed(Scancode::S) || kb.is_scancode_pressed(Scancode::Down) {
            dy += 1.0;
        }
        if kb.is_scancode_pressed(Scancode::A) || kb.is_scancode_pressed(Scancode::Left) {
            dx -= 1.0;
        }
        if kb.is_scancode_pressed(Scancode::D) || kb.is_scancode_pressed(Scancode::Right) {
            dx += 1.0;
        }
        // Normalize so diagonal movement isn't faster than cardinal movement.
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            dx /= len;
            dy /= len;
        }

        // ----- update -----
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32();
        last_frame = now;

        camera.x += dx * CAMERA_SPEED * dt;
        camera.y += dy * CAMERA_SPEED * dt;
        camera.clamp_to_world();

        for v in &mut vehicles {
            v.update(dt);
        }

        // ----- render -----
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        draw_visible_tiles(&mut canvas, &map, &camera);

        for v in &vehicles {
            draw_vehicle(&mut canvas, v, &camera);
        }

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }
}
