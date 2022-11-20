// #![warn(clippy::pedantic)]

use std::f32::consts::TAU;

use bevy::{
    ecs::component::TableStorage,
    prelude::{App, Commands, Component, CoreStage, Entity, NonSendMut, Plugin, Query, Without},
};
use knyst::{
    audio_backend::{CpalBackend, CpalBackendOptions},
    graph::{Gen, NodeAddress},
    prelude::{AudioBackend, GenContext, Graph, GraphSettings},
    Resources, ResourcesSettings,
};
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<AudioGraph>()
            .add_system_to_stage(CoreStage::PostUpdate, play_audio::<SineWave>);
    }
}

pub struct AudioGraph {
    pub(crate) graph: Graph,
    _backend: CpalBackend,
}

impl Default for AudioGraph {
    fn default() -> Self {
        let mut backend = CpalBackend::new(CpalBackendOptions::default())
            .unwrap_or_else(|err| panic!("Cannot initialize cpal backend. Error: {err}"));

        let sample_rate = backend.sample_rate() as f32;
        let block_size = backend.block_size().unwrap_or(64);

        let resources = Resources::new(ResourcesSettings {
            sample_rate,
            ..Default::default()
        });

        let mut graph = Graph::new(GraphSettings {
            num_outputs: backend.num_outputs(),
            block_size,
            sample_rate,
            ..Default::default()
        });

        backend
            .start_processing(&mut graph, resources)
            .unwrap_or_else(|err| panic!("Cannot start processing audio graph. Error: {err}"));

        Self {
            graph,
            _backend: backend,
        }
    }
}

impl AudioGraph {
    fn play_stream(&mut self, stream: impl Gen + Send + 'static) -> NodeAddress {
        let node_address = self.graph.push_gen(stream);
        self.graph
            .connect(node_address.to_graph_out())
            .unwrap_or_else(|err| panic!("Cannot connect stream to output. Error: {err}"));
        self.graph.commit_changes();
        self.graph.update();
        node_address
    }
}

pub struct Audio<T> {
    stream: T,
}

impl<T: Send + Sync + 'static> Component for Audio<T> {
    type Storage = TableStorage;
}

impl<T: Streamable> Audio<T> {
    pub fn new(stream: T) -> Self {
        Self { stream }
    }
}

#[derive(Component)]
pub struct AudioId(pub NodeAddress);

// #[derive(Deref, DerefMut)]
// pub struct AudioControl<T: Streamable>(Handle<T::Stream>);

// impl<T: Streamable> Component for AudioControl<T> {
//     type Storage = TableStorage;
// }

pub trait Streamable: Send + Sync + 'static {
    type Stream: Gen + Send;

    fn to_stream(&self) -> Self::Stream;
}

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
    fn process(&mut self, ctx: GenContext, resources: &mut Resources) -> knyst::prelude::GenState {
        let sample_rate = resources.sample_rate;
        self.generate_samples(sample_rate, ctx.outputs.get_channel_mut(0));
        knyst::prelude::GenState::Continue
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

fn play_audio<T: Streamable>(
    mut commands: Commands,
    audio_query: Query<(Entity, &Audio<T>), Without<AudioId>>,
    mut audio_graph: NonSendMut<AudioGraph>,
) {
    for (entity, audio) in audio_query.iter() {
        let stream = audio.stream.to_stream();
        let node_address = audio_graph.play_stream(stream);
        commands.entity(entity).insert(AudioId(node_address));
    }
}
