use nih_plug::prelude::*;
use std::sync::Arc;
use ring_buffer::RingBuffer;

mod ring_buffer;

#[derive(Enum, PartialEq)]
pub enum FilterType {
    FIR,
    IIR,
}

const MIN_GAIN: f32 = 0.0;
const MAX_GAIN: f32 = 1.0;
const MIN_DELAY: i32 = 0;
const MAX_DELAY: i32 = 100;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct Comb {
    params: Arc<CombParams>,
    delay_line: Vec<RingBuffer<f32>>
}

#[derive(Params)]
struct CombParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "delay"]
    pub delay: IntParam,
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "filter_type"]
    pub filter_type: EnumParam<FilterType>,
}

impl Default for Comb {
    fn default() -> Self {
        Self {
            params: Arc::new(CombParams::default()),
            delay_line: Vec::new()
        }
    }
}

impl Default for CombParams {
    fn default() -> Self {
        Self {
            delay: IntParam::new(
                "Delay", 
                MAX_DELAY/5,
                IntRange::Linear { 
                    min: MIN_DELAY, 
                    max: MAX_DELAY
                }
            )
            .with_unit(" ms"),
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                MAX_GAIN/2.0,
                FloatRange::Linear {
                    min: MIN_GAIN,
                    max: MAX_GAIN,
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            // .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            // .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            filter_type: EnumParam::new(
                "Filter Type", 
                FilterType::IIR
            )
        }
    }
}

impl Plugin for Comb {
    const NAME: &'static str = "Comb Filter";
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
        for _ in 0..channels{
            self.delay_line.push(RingBuffer::new((MAX_DELAY as f32 * sample_rate / 1000.0) as usize));
        }
        self.reset();
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
        let delay = self.params.delay.smoothed.next();
        for i in 0..self.delay_line.len(){
            self.delay_line[i].reset();
            if delay>0 {
                for _ in 0..(delay as f32 * self.delay_line[i].capacity() as f32 / 1000.0) as usize {
                    self.delay_line[i].push(0.0);
                }
            }
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sample_rate = context.transport().sample_rate;
        let capacity = (MAX_DELAY as f32 * sample_rate / 1000.0) as usize;
        for (index, channel_samples) in buffer.as_slice().iter_mut().enumerate() { // single channel buffer samples
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();
            let filter = self.params.filter_type.modulated_plain_value();
            let delay = self.params.delay.smoothed.next();
            if delay > 0 {
                let read_index = self.delay_line[index].get_read_index();
                self.delay_line[index].set_write_index((read_index + (delay as f32 * sample_rate/1000.0) as usize - 1) % capacity);
                match filter {
                    FilterType::FIR => {
                        for sample in channel_samples.iter_mut() {
                            let value: Option<f32> = self.delay_line[index].pop();
                            let input = *sample;
                            self.delay_line[index].push(input);
                            *sample = input + gain*value.unwrap_or(0.0);
                        }
                    },
                    FilterType::IIR => {
                        for sample in channel_samples.iter_mut() {
                            let value: Option<f32> = self.delay_line[index].pop();
                            let input = *sample;
                            *sample = input + gain * value.unwrap_or(0.0);
                            self.delay_line[index].push(*sample);
                        }
                    }
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Comb {
    const CLAP_ID: &'static str = "edu.gatech.ase-example";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple example plugin for Audio Software Engineering.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo, ClapFeature::Mono, ClapFeature::Surround, ClapFeature::Filter];
}

impl Vst3Plugin for Comb {
    const VST3_CLASS_ID: [u8; 16] = *b"ASE-2024-Example";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Comb);
nih_export_vst3!(Comb);
