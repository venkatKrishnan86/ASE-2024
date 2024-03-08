use nih_plug::prelude::*;
use rand::rngs::OsRng;
use std::sync::Arc;
use ring_buffer::RingBuffer;
use lfo::{Oscillator, LFO};

mod ring_buffer;
mod lfo;

#[derive(Enum, PartialEq)]
pub enum FilterType {
    FIR,
    IIR,
}

const MIN_WIDTH: i32 = 1;
const MAX_WIDTH: i32 = 1000;
const DEFAULT_WIDTH: i32 = 50;
const MIN_FREQ: f32 = 0.0;
const MAX_FREQ: f32 = 100.0;
const DEFAULT_FREQ: f32 = 5.0;

struct Vibrato {
    params: Arc<VibratoParams>,
    lfo: Vec<LFO>,
    delay_line: Vec<RingBuffer<f32>>
}

#[derive(Params)]
struct VibratoParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined.
    #[id = "modfreq"]
    pub mod_freq: FloatParam,
    #[id = "width"]
    pub width: IntParam,
    #[id = "lfo"]
    pub lfo: EnumParam<Oscillator>,
}

impl Default for Vibrato {
    fn default() -> Self {
        Self {
            params: Arc::new(VibratoParams::default()),
            lfo: Vec::new(),
            delay_line: Vec::new()
        }
    }
}

impl Default for VibratoParams {
    fn default() -> Self {
        Self {
            mod_freq: FloatParam::new(
                "Mod Frequency",
                DEFAULT_FREQ,
                FloatRange::Skewed { 
                    min: MIN_FREQ, 
                    max: MAX_FREQ, 
                    factor: 0.15 
                }
            )
            .with_smoother(SmoothingStyle::Logarithmic(1.5))
            .with_value_to_string(formatters::v2s_f32_rounded(2))
            .with_unit(" Hz"),
            width: IntParam::new(
                "Width",
                DEFAULT_WIDTH,
                IntRange::Linear { 
                    min: MIN_WIDTH,
                    max: MAX_WIDTH 
                }
            )
            .with_unit(" ms")
            .with_smoother(SmoothingStyle::Linear(1.0)),
            lfo: EnumParam::new(
                "LFO Type",
                Oscillator::Sine
            )
        }
    }
}

impl Plugin for Vibrato {
    const NAME: &'static str = "Vibrato";
    const VENDOR: &'static str = "Venkatakrishnan V K";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "vkrishnan65@gatech.edu";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        let sample_rate = buffer_config.sample_rate;
        let channels = audio_io_layout.main_input_channels.unwrap().get();
        let lfo_type = self.params.lfo.default_plain_value();
        for _ in 0..channels{
            self.delay_line.push(RingBuffer::new(2 + MAX_WIDTH as usize * 3));
            self.lfo.push(lfo::LFO::new(sample_rate as u32, sample_rate as usize * 2, lfo_type.clone(), DEFAULT_FREQ))
        }
        self.reset();
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
        for channel in self.delay_line.iter_mut(){
            channel.set_read_index(0);
            channel.set_write_index(1 + MAX_WIDTH as usize * 3);
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let capacity = 2 + MAX_WIDTH as usize * 3;
        for (channel, channel_samples) in buffer.as_slice().iter_mut().enumerate() { // single channel buffer samples
            // Smoothing is optionally built into the parameters themselves
            let lfo_type = self.params.lfo.modulated_plain_value();
            self.lfo[channel].set_oscillator(lfo_type);

            let width = self.params.width.smoothed.next();
            let read_index = self.delay_line[channel].get_read_index();
            self.delay_line[channel].set_write_index((read_index + (1 + width as usize*3)) % capacity);

            let frequency = self.params.mod_freq.smoothed.next();
            let _ = self.lfo[channel].set_frequency(frequency);
            for sample in channel_samples.iter_mut() {
                let modulator = self.lfo[channel].get_sample();
                let offset = 1.0 + width as f32 + width as f32 * modulator;
                let _ = self.delay_line[channel].pop();
                self.delay_line[channel].push(*sample);
                *sample = self.delay_line[channel].get_frac(offset);
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Vibrato {
    const CLAP_ID: &'static str = "edu.gatech.ase-example";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A Vibrato Plugin.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo, ClapFeature::Mono, ClapFeature::Surround, ClapFeature::Filter];
}

impl Vst3Plugin for Vibrato {
    const VST3_CLASS_ID: [u8; 16] = *b"A Vibrato Plugin";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Vibrato);
nih_export_vst3!(Vibrato);
