use engine::platform::tilemap::{SCREEN_WIDTH, SCREEN_HEIGHT, Tilemap, draw_visible_tiles, draw_vehicle, draw_traffic_signals};
use engine::traffic::{TrafficSignals};
use engine::platform::camera::{Camera, CAMERA_SPEED};
use engine::vehicle::{Vehicles};
use engine::scripting::environment::{ScriptingEnvironment};
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Scancode};
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{Canvas, FRect, Texture, TextureCreator};
use sdl3::video::{Window, WindowContext};
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
    let texture_creator = canvas.texture_creator();
    let texture = create_vehicle_texture(&mut canvas, &texture_creator, Color::RGB( 60, 220,  80));
    let mut traffic_signals = TrafficSignals::new(&tile_map.map);
    let mut scripting_environment = ScriptingEnvironment::new(&mut traffic_signals);
    scripting_environment.start();
    let mut last_frame = Instant::now();
    let mut spawn_timer = 0.0;

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

        spawn_timer += dt;
        if spawn_timer > 1.0 {
            vehicles.spawn_vehicle();
            spawn_timer = 0.0;
        }
        vehicles.update(&traffic_signals, &tile_map, dt);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        draw_visible_tiles(&mut canvas, &tile_map.map, &camera);
        for vehicle in &vehicles.inventory {
            draw_vehicle(&mut canvas, &vehicle, &texture, &camera);
        }

        draw_traffic_signals(&mut canvas, &traffic_signals.traffic_signals.borrow(), &camera);

        scripting_environment.update();
        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }
}

pub fn create_vehicle_texture<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    body_color: Color,
) -> Texture<'a> {
    let (w, h) = (16, 24);
    let mut texture = texture_creator
        .create_texture_target(PixelFormat::RGBA8888, w, h)
        .unwrap();

    canvas.with_texture_canvas(&mut texture, |c| {
        c.set_draw_color(Color::RGBA(0, 0, 0, 0));
        c.clear();
        c.set_draw_color(body_color);
        c.fill_rect(FRect::new(0.0, 0.0, w as f32, h as f32)).unwrap();
        c.set_draw_color(Color::RGB(40, 60, 80));
        c.fill_rect(FRect::new(2.0, 4.0, 12.0, 6.0)).unwrap();
    }).unwrap();

    texture
}
