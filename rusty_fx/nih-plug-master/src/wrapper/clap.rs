#[macro_use]
mod util;

mod context;
mod descriptor;
mod factory;
pub mod features;
mod wrapper;

/// Re-export for the wrapper.
pub use self::factory::Factory;
pub use clap_sys::entry::clap_plugin_entry;
pub use clap_sys::factory::plugin_factory::CLAP_PLUGIN_FACTORY_ID;
pub use clap_sys::version::CLAP_VERSION;
pub use lazy_static::lazy_static;

/// Export a CLAP plugin from this library using the provided plugin type.
#[macro_export]
macro_rules! nih_export_clap {
    ($plugin_ty:ty) => {
        // We need a function pointer to a [wrapper::get_factory()] that creates a factory for `$plugin_ty`, so we need to generate the function inside of this macro
        #[doc(hidden)]
        mod clap {
            // Because `$plugin_ty` is likely defined in the enclosing scope
            use super::*;

            // We don't use generics inside of statics, so this lazy_static is used as kind of an
            // escape hatch
            ::nih_plug::wrapper::clap::lazy_static! {
                static ref FACTORY: ::nih_plug::wrapper::clap::Factory<$plugin_ty> = ::nih_plug::wrapper::clap::Factory::default();
            }

            pub extern "C" fn init(_plugin_path: *const ::std::os::raw::c_char) -> bool {
                ::nih_plug::wrapper::setup_logger();
                true
            }

            pub extern "C" fn deinit() {}

            pub extern "C" fn get_factory(
                factory_id: *const ::std::os::raw::c_char,
            ) -> *const ::std::ffi::c_void {
                if !factory_id.is_null()
                    && unsafe { ::std::ffi::CStr::from_ptr(factory_id) }
                        == ::nih_plug::wrapper::clap::CLAP_PLUGIN_FACTORY_ID
                {
                    &*FACTORY as *const _ as *const ::std::ffi::c_void
                } else {
                    std::ptr::null()
                }
            }
        }

        /// The CLAP plugin's entry point.
        #[no_mangle]
        #[used]
        pub static clap_entry: ::nih_plug::wrapper::clap::clap_plugin_entry =
            ::nih_plug::wrapper::clap::clap_plugin_entry {
                clap_version: ::nih_plug::wrapper::clap::CLAP_VERSION,
                init: Some(self::clap::init),
                deinit: Some(self::clap::deinit),
                get_factory: Some(self::clap::get_factory),
            };
    };
}
