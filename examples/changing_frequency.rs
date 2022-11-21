use bevy::{
    prelude::{App, Commands, Query, Res},
    time::Time,
    DefaultPlugins,
};
use bevy_prototype_audio_graph::{sine::SineWave, Audio, AudioControl, AudioPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(play_sound)
        .add_system(change_frequency)
        .run()
}

fn play_sound(mut commands: Commands) {
    commands.spawn(Audio::new(SineWave {
        frequency_hz: 440.0,
        phase: 0.0,
    }));
}

fn change_frequency(q_control: Query<&AudioControl<SineWave>>, time: Res<Time>) {
    if let Ok(control) = q_control.get_single() {
        let exp = time.elapsed_seconds_wrapped().sin();
        let frequency_hz = 2.0_f32.powf(exp) * 440.0;
        control.set_frequency(frequency_hz);
    }
}
