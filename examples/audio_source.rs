use std::{io::Cursor, sync::Arc};

use bevy::{
    prelude::{App, Commands, Entity, Resource},
    DefaultPlugins,
};
use bevy_prototype_audio_graph::{source::AudioSource, Audio, AudioPlugin};
use symphonia::core::{
    audio::{AudioBuffer, AudioBufferRef, Signal},
    conv::{FromSample, IntoSample},
    io::MediaSourceStream,
    sample::Sample,
};

static WINDLESS_SLOPES: &[u8] = include_bytes!("../assets/Windless Slopes.ogg");

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
        .spawn(Audio::new(AudioSource {
            frames: decode(WINDLESS_SLOPES.to_vec()),
        }))
        .id();
    commands.insert_resource(SoundEntity(entity));
}

// basically copy pasted from kira
// sorry tesselode

fn decode(bytes: Vec<u8>) -> Arc<[[f32; 2]]> {
    let codecs = symphonia::default::get_codecs();
    let probe = symphonia::default::get_probe();
    let mss = MediaSourceStream::new(Box::new(Cursor::new(bytes)), Default::default());
    let mut format_reader = probe
        .format(
            &Default::default(),
            mss,
            &Default::default(),
            &Default::default(),
        )
        .unwrap()
        .format;
    let codec_params = &format_reader.default_track().unwrap().codec_params;
    let mut decoder = codecs.make(codec_params, &Default::default()).unwrap();
    let mut frames = vec![];
    loop {
        match format_reader.next_packet() {
            Ok(packet) => {
                let buffer = decoder.decode(&packet).unwrap();
                load_frames_from_buffer_ref(&mut frames, &buffer);
            }
            Err(error) => match error {
                symphonia::core::errors::Error::IoError(error) => {
                    if error.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    }
                    panic!();
                }
                _ => panic!(),
            },
        }
    }
    frames.into()
}

fn load_frames_from_buffer_ref(frames: &mut Vec<[f32; 2]>, buffer: &AudioBufferRef) {
    match buffer {
        AudioBufferRef::U8(buffer) => load_frames_from_buffer(frames, buffer),
        AudioBufferRef::U16(buffer) => load_frames_from_buffer(frames, buffer),
        AudioBufferRef::U24(buffer) => load_frames_from_buffer(frames, buffer),
        AudioBufferRef::U32(buffer) => load_frames_from_buffer(frames, buffer),
        AudioBufferRef::S8(buffer) => load_frames_from_buffer(frames, buffer),
        AudioBufferRef::S16(buffer) => load_frames_from_buffer(frames, buffer),
        AudioBufferRef::S24(buffer) => load_frames_from_buffer(frames, buffer),
        AudioBufferRef::S32(buffer) => load_frames_from_buffer(frames, buffer),
        AudioBufferRef::F32(buffer) => load_frames_from_buffer(frames, buffer),
        AudioBufferRef::F64(buffer) => load_frames_from_buffer(frames, buffer),
    }
}

fn load_frames_from_buffer<S: Sample>(frames: &mut Vec<[f32; 2]>, buffer: &AudioBuffer<S>)
where
    f32: FromSample<S>,
{
    match buffer.spec().channels.count() {
        1 => {
            for sample in buffer.chan(0) {
                let sample = (*sample).into_sample();
                frames.push([sample, sample]);
            }
        }
        2 => {
            for (left, right) in buffer.chan(0).iter().zip(buffer.chan(1).iter()) {
                frames.push([(*left).into_sample(), (*right).into_sample()]);
            }
        }
        _ => panic!(),
    }
}
