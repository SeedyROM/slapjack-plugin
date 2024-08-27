use faust_state::{DspHandle, StateHandle};
use nih_plug::prelude::*;
use std::sync::Arc;

use faust::Slapjack;

#[allow(unexpected_cfgs, static_mut_refs)]
mod faust {
    include!(concat!(env!("OUT_DIR"), "/dsp.rs"));
}

struct FaustIntegration {
    params: Arc<FaustIntegrationParams>,
    dsp: DspHandle<Slapjack>,
    state: StateHandle,
}

#[derive(Params)]
struct FaustIntegrationParams {
    #[id = "slapback_delay"]
    pub slapback_delay: FloatParam,
}

impl Default for FaustIntegration {
    fn default() -> Self {
        let (dsp, state) = DspHandle::<Slapjack>::new();

        Self {
            params: Arc::new(FaustIntegrationParams::default()),
            dsp,
            state,
        }
    }
}

impl Default for FaustIntegrationParams {
    fn default() -> Self {
        Self {
            slapback_delay: FloatParam::new(
                "Slapback Delay",
                100.0,
                FloatRange::Linear {
                    min: 10.0,
                    max: 500.0,
                },
            )
            .with_unit("ms"),
        }
    }
}

impl Plugin for FaustIntegration {
    const NAME: &'static str = "Slapjack";
    const VENDOR: &'static str = "SeedyROM (Zack Kollar)";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "me@seedyrom.io";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        // Stereo
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        // Mono
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sample_rate = buffer_config.sample_rate;
        self.dsp.init(sample_rate as i32);
        true
    }

    fn reset(&mut self) {
        // // On reset we need to reinitialize the DSP and state
        // let (dsp, state) = DspHandle::<Slapjack>::new();
        // self.dsp = dsp;
        // self.state = state;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let slapback_delay = self.params.slapback_delay.smoothed.next();

        // Update the faust DSP state
        self.state
            .set_by_path("3. Slapback/Time", slapback_delay)
            .unwrap();
        self.state.send();

        let num_samples = buffer.samples() as i32;
        // SAFETY: This transmute is safe because we're only changing mutability,
        // not the shape or size of the slice. We're promising not to modify the inputs.
        let inputs: &[&[f32]] = unsafe { std::mem::transmute(buffer.as_slice_immutable()) };
        let outputs: &mut [&mut [f32]] = buffer.as_slice();

        self.dsp.update_and_compute(num_samples, inputs, outputs);

        ProcessStatus::Normal
    }
}

impl ClapPlugin for FaustIntegration {
    const CLAP_ID: &'static str = "com.seedyrom.slapjack";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("A simple plugin that processes audio using a Faust program called slapjack.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Mono,
        ClapFeature::Stereo,
        ClapFeature::MultiEffects,
        ClapFeature::Custom("Fuckery"),
    ];
}

impl Vst3Plugin for FaustIntegration {
    const VST3_CLASS_ID: [u8; 16] = *b"SeedyROMSlapjack";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Custom("Fuckery"),
        Vst3SubCategory::Modulation,
        Vst3SubCategory::Dynamics,
        Vst3SubCategory::Reverb,
        Vst3SubCategory::Delay,
    ];
}

nih_export_clap!(FaustIntegration);
nih_export_vst3!(FaustIntegration);
