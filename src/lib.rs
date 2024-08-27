use faust_state::{DspHandle, StateHandle};
use nih_plug::prelude::*;
use std::sync::Arc;

use faust::Slapjack;

#[allow(unexpected_cfgs, static_mut_refs)]
mod faust {
    include!(concat!(env!("OUT_DIR"), "/dsp.rs"));
}

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct FaustIntegration {
    params: Arc<FaustIntegrationParams>,
    dsp: DspHandle<Slapjack>,
    state: StateHandle,
}

#[derive(Params)]
struct FaustIntegrationParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
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
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sample_rate = buffer_config.sample_rate;
        self.dsp.init(sample_rate as i32);
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
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
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Integrate faust and nih_plug");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for FaustIntegration {
    const VST3_CLASS_ID: [u8; 16] = *b"SeedyROMSlapjack";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(FaustIntegration);
nih_export_vst3!(FaustIntegration);
