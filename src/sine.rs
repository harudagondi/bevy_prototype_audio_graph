use std::f32::consts::TAU;

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
    frequency: f32,
    phase: f32,
}

impl SineWaveStream {
    fn seek_to(&mut self, t: f32) {
        self.phase = (self.phase + t * self.frequency) % TAU;
    }

    fn generate_samples(&mut self, sample_rate: f32, out: &mut [f32]) {
        let interval = 1.0 / sample_rate;
        for (i, x) in out.iter_mut().enumerate() {
            let t = interval * i as f32;
            *x = (t * self.frequency + self.phase).sin();
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

impl Streamable for SineWave {
    type Stream = SineWaveStream;

    fn to_stream(&self) -> Self::Stream {
        SineWaveStream {
            frequency: self.frequency_hz * TAU,
            phase: self.phase,
        }
    }
}
