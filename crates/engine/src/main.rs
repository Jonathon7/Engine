use engine::platform::tilemap::{SCREEN_WIDTH, SCREEN_HEIGHT, Tilemap, draw_visible_tiles, draw_vehicle, draw_traffic_signals};
use engine::traffic::{TrafficSignals};
use engine::platform::camera::{Camera, CAMERA_SPEED};
use engine::vehicle::{Vehicles};
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Scancode};
use sdl3::pixels::Color;
use std::time::{Duration, Instant};

fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let window = video_subsystem
        .window("Traffic Manager Demo", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let tile_map = Tilemap::new();
    let mut camera = Camera { x: 0.0, y: 0.0 };
    let mut vehicles = Vehicles::new(&tile_map.map);
    let traffic_signals = TrafficSignals::new(&tile_map.map);

    let mut last_frame = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

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

        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            dx /= len;
            dy /= len;
        }

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32();
        last_frame = now;

        camera.x += dx * CAMERA_SPEED * dt;
        camera.y += dy * CAMERA_SPEED * dt;
        camera.clamp_to_world();

        vehicles.spawn_vehicle();
        vehicles.update(&traffic_signals, &tile_map, dt);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        draw_visible_tiles(&mut canvas, &tile_map.map, &camera);
        for vehicle in &vehicles.inventory {
            draw_vehicle(&mut canvas, &vehicle, &camera);
        }

        draw_traffic_signals(&mut canvas, &traffic_signals.traffic_signals, &camera);

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }
}
