use std::sync::Arc;

use knyst::{
    graph::Gen,
    prelude::{GenContext, GenState},
    Resources,
};

use crate::Streamable;

pub struct AudioSource {
    pub frames: Arc<[[f32; 2]]>,
}

pub struct AudioSourceStream {
    frames: Arc<[[f32; 2]]>,
    cursor: usize,
}

impl Gen for AudioSourceStream {
    fn process(&mut self, ctx: GenContext, _: &mut Resources) -> GenState {
        for (out_left, out_right) in ctx
            .outputs
            .get_channel_mut(0)
            .iter_mut()
            .zip(ctx.outputs.get_channel_mut(1))
        {
            let [left, right] = self.frames.get(self.cursor).copied().unwrap_or([0.0, 0.0]);
            *out_left = left;
            *out_right = right;
            self.cursor += 1
        }

        if self.cursor < self.frames.len() {
            GenState::Continue
        } else {
            GenState::FreeSelf
        }
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        2
    }
}

pub struct AudioSourceControl;

impl Streamable for AudioSource {
    type Stream = AudioSourceStream;
    type Control = AudioSourceControl;

    fn to_stream(&self) -> (Self::Stream, Self::Control) {
        (
            AudioSourceStream {
                frames: self.frames.clone(),
                cursor: 0,
            },
            AudioSourceControl,
        )
    }
}
