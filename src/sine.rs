use std::{
    f32::consts::TAU,
    sync::{atomic::Ordering, Arc},
};

use atomic_float::AtomicF32;
use knyst::{
    graph::Gen,
    prelude::{GenContext, GenState},
    Resources,
};

use crate::Streamable;

pub struct SineWave {
    pub frequency_hz: f32,
    pub phase: f32,
}

pub struct SineWaveStream {
    frequency: Arc<AtomicF32>,
    phase: f32,
}

impl SineWaveStream {
    fn seek_to(&mut self, t: f32) {
        self.phase = (self.phase + t * self.frequency.load(Ordering::Relaxed)) % TAU;
    }

    fn generate_samples(&mut self, sample_rate: f32, out: &mut [f32]) {
        let interval = 1.0 / sample_rate;
        for (i, x) in out.iter_mut().enumerate() {
            let t = interval * i as f32;
            *x = (t * self.frequency.load(Ordering::Relaxed) + self.phase).sin();
        }
        self.seek_to(interval * out.len() as f32);
    }
}

impl Gen for SineWaveStream {
    fn process(&mut self, ctx: GenContext, resources: &mut Resources) -> GenState {
        let sample_rate = resources.sample_rate;
        self.generate_samples(sample_rate, ctx.outputs.get_channel_mut(0));
        GenState::Continue
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }
}

pub struct SineWaveControl {
    frequency: Arc<AtomicF32>,
}

impl SineWaveControl {
    pub fn frequency(&self) -> f32 {
        self.frequency.load(Ordering::Relaxed) / TAU
    }

    pub fn set_frequency(&self, frequency_hz: f32) {
        self.frequency.store(frequency_hz * TAU, Ordering::Relaxed);
    }
}

impl Streamable for SineWave {
    type Stream = SineWaveStream;
    type Control = SineWaveControl;

    fn to_stream(&self) -> (Self::Stream, Self::Control) {
        let frequency = Arc::new(AtomicF32::new(self.frequency_hz * TAU));
        let control = SineWaveControl {
            frequency: frequency.clone(),
        };
        let stream = SineWaveStream {
            frequency,
            phase: self.phase,
        };
        (stream, control)
    }
}
