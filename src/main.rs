use rand::prelude::*;
use rusty_engine::prelude::*;

struct GameState {
    marble_labels: Vec<String>,
    cars_left: i32,
    spawn_timer: Timer,
}

const MARBLE_SPEED: f32 = 600.0;
const CAR_SPEED: f32 = 250.0;

fn main() {
    let mut game = Game::new();

    let game_state = GameState {
        marble_labels: vec!["marble1".into(), "marble2".into(), "marble3".into()],
        cars_left: 25,
        spawn_timer: Timer::from_seconds(0.0, false),
    };

    // Set the title of the window
    game.window_settings(WindowDescriptor {
        title: "Car shot".into(),
        ..Default::default()
    });

    // Play some background music
    game.audio_manager.play_music(MusicPreset::Classy8Bit, 0.1);

    // Create a player
    let player = game.add_sprite("player", SpritePreset::RacingBarrierRed);
    player.rotation = UP;
    player.scale = 0.5;
    player.translation.y = -325.0;
    player.layer = 10.0;

    // Create a Left car text
    let cars_left = game.add_text("cars_left", format!("Cars left: {}", game_state.cars_left));
    cars_left.translation = Vec2::new(540.0, -320.0);

    game.add_logic(game_logic);
    game.run(game_state);
}

fn game_logic(engine: &mut Engine, game_state: &mut GameState) {
    // handle marble gun movement
    let player = engine.sprites.get_mut("player").unwrap();
    if let Some(location) = engine.mouse_state.location() {
        player.translation.x = location.x;
    }
    let player_x = player.translation.x;

    // Shoot marbles
    if engine.mouse_state.just_pressed(MouseButton::Left) {
        if let Some(label) = game_state.marble_labels.pop() {
            let marble = engine.add_sprite(label, SpritePreset::RollingBallBlue);
            marble.translation.x = player_x;
            marble.translation.y = -275.0;
            marble.layer = 5.0;
            marble.collision = true;
            engine.audio_manager.play_sfx(SfxPreset::Impact2, 0.2);
        }
    }

    // Move marbles
    engine
        .sprites
        .values_mut()
        .filter(|sprite| sprite.label.starts_with("marble"))
        .for_each(|marble| marble.translation.y += MARBLE_SPEED * engine.delta_f32);

    // Move cars accros the screen
    engine
        .sprites
        .values_mut()
        .filter(|sprite| sprite.label.starts_with("car"))
        .for_each(|car| car.translation.x += CAR_SPEED * engine.delta_f32);

    // Clean up sprites that have gone off the screen
    let mut labels_to_delete = Vec::new();
    for (label, sprite) in engine.sprites.iter() {
        if sprite.translation.y > 400.0 || sprite.translation.x > 750.0 {
            labels_to_delete.push(label.clone());
        }
    }

    for label in labels_to_delete {
        engine.sprites.remove(&label);
        if label.starts_with("marble") {
            game_state.marble_labels.push(label);
        }
    }

    // Spawn cars
    if game_state.spawn_timer.tick(engine.delta).just_finished() {
        // Reset the timer to the new value
        game_state.spawn_timer = Timer::from_seconds(thread_rng().gen_range(0.1..1.25), false);
        // Get the new car
        if game_state.cars_left > 0 {
            game_state.cars_left -= 1;
            let text = engine.texts.get_mut("cars_left").unwrap();
            text.value = format!("Cars left: {}", game_state.cars_left);
            let label = format!("car{}", game_state.cars_left);
            use SpritePreset::*;
            let car_choices = vec![
                RacingCarBlack,
                RacingCarBlue,
                RacingCarRed,
                RacingCarYellow,
                RacingCarGreen,
            ];
            let sprite_preset = car_choices
                .iter()
                .choose(&mut thread_rng())
                .unwrap()
                .clone();
            let car = engine.add_sprite(label, sprite_preset);
            car.translation.x = -740.0;
            car.translation.y = thread_rng().gen_range(-100.0..325.0);
            car.collision = true;
        }
    }

    // Handle collisions
    for event in engine.collision_events.drain(..) {
        if event.state.is_end() {
            continue;
        }
        if !event.pair.one_starts_with("marble") {
            continue;
        }

        for label in event.pair {
            engine.sprites.remove(&label);
            if label.starts_with("marble") {
                game_state.marble_labels.push(label);
            }
            engine.audio_manager.play_sfx(SfxPreset::Confirmation1, 0.2);
        }
    }
}
