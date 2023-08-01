
use nih_plug::{params::*, prelude::*};
use nih_plug_egui::EguiState;
use std::sync::Arc;

pub static MAX_PREDELAY: i32 = 100;

#[derive(Params)]
pub(crate) struct ReverbPluginParams {

    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    // gain for the processed signal
    #[id = "wet"]
    pub wet: FloatParam,

    // gain for the unprocessed signal
    #[id = "dry"]
    pub dry: FloatParam,

    // pre_delay before the wet signal kicks in
    #[id = "pre_delay"]
    pub pre_delay: IntParam,
    
    // Size of the emulated room
    #[id = "size"]
    pub size: FloatParam,

    // The extent to which the left and right channels are mixed in the output
    #[id = "stereo_width"]
    pub stereo_width: FloatParam,

    // Damping factor of the walls of the emulated room
    // higher values cause more high frequence attenuation
    #[id = "damping"]
    pub damping: FloatParam
}


impl Default for ReverbPluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(500, 320),
            wet: FloatParam::new(
                "Wet",
                util::db_to_gain( 0.0 ),
                FloatRange::Skewed {
                    min:  util::db_to_gain( -60.0 ),
                    max: util::db_to_gain( 0.0 ),
                    factor: FloatRange::gain_skew_factor( -60.0, 0.0 )
                }
            )
            .with_smoother( SmoothingStyle::Logarithmic( 10.0 ) )
            .with_unit( " dB" )
            .with_value_to_string( formatters::v2s_f32_gain_to_db( 2 ) )
            .with_string_to_value( formatters::s2v_f32_gain_to_db() ),

            dry: FloatParam::new(
                "Dry",
                util::db_to_gain( -60.0 ),
                FloatRange::Skewed {
                    min:  util::db_to_gain( -60.0 ),
                    max: util::db_to_gain( 0.0 ),
                    factor: FloatRange::gain_skew_factor( -60.0, 0.0 )
                }
            )
            .with_smoother( SmoothingStyle::Logarithmic( 10.0 ) )
            .with_unit( " dB" )
            .with_value_to_string( formatters::v2s_f32_gain_to_db( 2 ) )
            .with_string_to_value( formatters::s2v_f32_gain_to_db() ),

           pre_delay: IntParam::new(
            "Pre delay",
            0,
            IntRange::Linear { min: 0, max: MAX_PREDELAY }
           )
           .non_automatable()
           .with_unit( " ms" ),

           size: FloatParam::new(
            "Room size",
            0.5,
            FloatRange::Linear { min: 0f32, max: 1f32 }
           )
           .with_smoother( SmoothingStyle::Linear( 10.0 ) ),

           stereo_width: FloatParam::new(
            "Stereo Width",
            0f32,
            FloatRange::Linear { min: 0f32, max: 1f32 }
           )
           .with_smoother( SmoothingStyle::Linear( 10.0 ) ),

           damping: FloatParam::new(
            "Damping",
            0.5,
            FloatRange::Linear {
                min: util::db_to_gain( -60.0 ).sqrt(),
                max: 1f32
            })
            .with_smoother( SmoothingStyle::Exponential( 10.0 ) )
            .with_value_to_string( formatters::v2s_f32_rounded( 2 ) ),
        }
    }
}
