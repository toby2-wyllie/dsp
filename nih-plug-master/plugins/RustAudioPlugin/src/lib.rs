mod delay;
mod parameters;

use nih_plug::{prelude::*};
use nih_plug_egui::{create_egui_editor, widgets, egui::CentralPanel};
use std::sync::Arc;
use freeverb::Freeverb;
use delay::StereoDelay;
use nih_plug_egui::egui::{Layout, Align, emath::Vec2, Visuals};
use egui_extras::RetainedImage;
use parameters::*;

// This is the plugin's state, it will be accessible by the processing and
// GUI threads
struct AudioPlugin {
    params: Arc<ReverbPluginParams>,
    reverb: Freeverb,
    pre_delay: StereoDelay,
    sample_rate: f32
}

impl Default for AudioPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new( ReverbPluginParams::default() ),
            reverb: Freeverb::new( 44100 ),
            pre_delay: StereoDelay::new( 0, MAX_PREDELAY, 44100.0 ),
            sample_rate: 44100.0
        }
    }
}

impl Plugin for AudioPlugin {
    const NAME: &'static str = "Rust Reverb";
    const VENDOR: &'static str = "Me";

    const URL: &'static str = "http://examples.com";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),

            aux_input_ports: &[],
            aux_output_ports: &[],

            names: PortNames::const_default(),
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {

        // Apply the parameter updates to the processing elements
        match self.apply_parameters() {
            Ok( _ ) => (),
            Err( msg ) => return ProcessStatus::Error( msg )
        };

        // process the samples in the buffer
        for channel_samples in buffer.iter_samples() {

            let mut channel_iter = channel_samples.into_iter();
            
            let left_opt = channel_iter.next();
            let right_opt = channel_iter.next();

            match ( left_opt, right_opt ) {
                ( Some( left ), Some( right ) ) => {
                    let reverb_sample = self.reverb.tick( ( *left as f64, *right as f64 ) );
                    let reverb_sample = self.predelay( reverb_sample );

                    *left = reverb_sample.0 as f32 + *left * self.params.dry.value();
                    *right = reverb_sample.1 as f32 + *right * self.params.dry.value();
                },
                ( Some( left ), None ) =>{
                    let reverb_sample = self.reverb.tick( ( *left as f64, 0f64 ) );
                    let reverb_sample = self.predelay( reverb_sample );

                    *left = reverb_sample.0 as f32 + *left * self.params.dry.value();
                } ,
                _ => return ProcessStatus::Error( "Unsupported channel configuration!" )
            }
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}

    fn initialize(
            &mut self,
            _audio_io_layout: &AudioIOLayout,
            buffer_config: &BufferConfig,
            _context: &mut impl InitContext<Self>,
        ) -> bool {
            self.reverb = Freeverb::new( buffer_config.sample_rate as usize );
            self.reverb.set_dry( 0f64 );
            let _ = self.apply_parameters();
            
            self.sample_rate = buffer_config.sample_rate;

            self.pre_delay = StereoDelay::new(
                self.params.pre_delay.value(),
                MAX_PREDELAY,
                buffer_config.sample_rate );

            true
    }

    fn editor(
        &mut self,
        _async_executor: AsyncExecutor<Self>
        ) -> Option<Box<dyn Editor>> {

            let params = self.params.clone();
            create_egui_editor(self.params.editor_state.clone(), (), |_, _| {},
            move | ctx, setter, _ | {
                ctx.set_visuals( Visuals::dark() );

                CentralPanel::default().show(ctx, | ui | {

                    let ferris = RetainedImage::from_svg_str(
                        "Ferris", include_str!("assets/rustacean-flat-happy.svg") );        

                    ui.allocate_ui_with_layout(
                        Vec2::new( 500.0, 50.0 ),
                        Layout::left_to_right( Align::Center ),
                        | ui | {
                            if let Ok( img ) = ferris {
                                ui.image(
                                    img.texture_id( ctx ),
                                    Vec2::new( 75.0, 50.0 )
                                );

                                ui.separator();
                            }

                            ui.centered_and_justified( | ui | {
                                ui.heading( "Rust Reverb" );
                            } );
                        });

                    ui.separator();

                    ui.label( params.size.name() );
                    ui.add( widgets::ParamSlider::for_param( &params.size, setter ) );

                    ui.label( params.damping.name() ); 
                    ui.add( widgets::ParamSlider::for_param( &params.damping, setter ) );

                    ui.label( params.stereo_width.name() );
                    ui.add( widgets::ParamSlider::for_param( &params.stereo_width, setter ) );

                    ui.label( params.pre_delay.name() );
                    ui.add( widgets::ParamSlider::for_param( &params.pre_delay, setter ) );

                    ui.label( params.dry.name() );
                    ui.add( widgets::ParamSlider::for_param( &params.dry, setter ) );

                    ui.label( params.wet.name() );
                    ui.add( widgets::ParamSlider::for_param( &params.wet, setter ) );
                });
            })
    }
}

impl AudioPlugin {
    fn predelay( &mut self, sample: ( f64, f64 ) ) -> ( f64, f64 ) {
        match self.pre_delay.length() {
            0 | 1 => {
                sample
            },
            _ => {
                let delayed_sample = self.pre_delay.head();
                self.pre_delay.consume( sample );
                delayed_sample
            }
        }
    }

    fn apply_parameters( &mut self ) -> Result<&'static str, &'static str> {
        // update parameter values
        self.reverb.set_wet( self.params.wet.value() as f64 );
        self.reverb.set_width( self.params.stereo_width.value() as f64 );
        self.reverb.set_room_size( self.params.size.value() as f64 );
        self.reverb.set_dampening( self.params.damping.value() as f64 );

        self.pre_delay.set_delay( self.params.pre_delay.value() )
    }
}

impl Vst3Plugin for AudioPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"d407329a7ff424b6";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Reverb];
}

nih_export_vst3!(AudioPlugin);

