use bevy::{
    prelude::{App, Commands, Entity, Resource},
    DefaultPlugins,
};
use bevy_prototype_audio_graph::{Audio, AudioPlugin, SineWave};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(play_sound)
        .run()
}

#[derive(Resource)]
struct SoundEntity(Entity);

fn play_sound(mut commands: Commands) {
    let entity = commands
        .spawn(Audio::new(SineWave {
            frequency_hz: 440.0,
            phase: 0.0,
        }))
        .id();
    commands.insert_resource(SoundEntity(entity));
}
