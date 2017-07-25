//! # Rubrail
//!
//! Rubrail is a library for building persistent, always-available UIs for the
//! Mac Touch Bar.
//!
//! Rubrail uses **private** Apple APIs to add an expandable icon to the Touch
//! Bar's "Control Strip" region, the section that is, by default, always
//! displayed on the right side of the bar.  This lets you create a Touch Bar
//! menu that is always available, regardless of which app is currently in
//! focus.  It supports a variety of common UI elements, including recursive
//! "popbar" submenus, and uses callbacks to your own functions or closures to
//! inform your app of touch interactions.
//!
//! # Getting Rubrail
//!
//! [Cargo Crate](https://crates.io/crates/rubrail)
//!
//! [GitHub Source](https://github.com/mrmekon/rubrail-rs)
//!
//! # Execution Environment
//!
//! Rubrail can only communicate with Apple's Touch Bar service when it is
//! executed from a Mac Application environment: a valid _.app_ bundle with an
//! _Info.plist_ and the correct directory structure.
//!
//! When run outside of an app bundle, like when running with `cargo run`,
//! Rubrail will work correctly behind the scenes, but your Touch Bar will not
//! be registered with the system and will not display on the Control Strip,
//! making it inaccessible to the user.
//!
//! The examples are bundled with a script to generate a minimal valid app
//! bundle, and a wrapper example to move the real example into a bundle and
//! execute it.  You can execute the examples correctly with this comand:
//!
//! `$ cargo test && cargo run --example example_launcher`
//!
//! # Memory Management
//!
//! Rubrail manually manages the lifecycle of allocated Foundation and Cocoa
//! elements (the Objective-C classes that provide the UI features).  Because of
//! this, it is possible to leak memory.  Care should be taken to use the API
//! as intended to avoid leaks.
//!
//! The internal memory allocation strategy is to freely allocate objects
//! whenever a _create*()_ function is called, to associate all allocated
//! objects with a bar, and to register that bar with the system.  If the bar
//! is replaced (by registering a new bar with the sytem), the replacement
//! logic is responsible for recursively deallocating all items associated with
//! the bar and its subbars, and then deallocating the bar itself.
//!
//! Any objects created with a _create*()_ function that are never added to a
//! bar that is set as the system bar will be leaked.
//!
#![deny(missing_docs)]

#[allow(dead_code)]
#[allow(unused_variables)]
mod interface;
pub use interface::*;

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
#[macro_use]
mod wrapper;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
#[macro_use]
extern crate objc;

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
#[macro_use]
mod touchbar;

/// Main controller for creating and using Touch Bar UIs
///
/// Blah
#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
pub use touchbar::Touchbar as Touchbar;

#[cfg(not(feature = "private_api"))]
mod dummy;

/// Main controller for creating and using Touch Bar UIs
///
/// Blah
#[cfg(not(feature = "private_api"))]
pub use dummy::DummyTouchbar as Touchbar;

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
pub use touchbar::util;

#[cfg(not(feature = "private_api"))]
pub use dummy::util;

/// Module for creating and running a simple Mac application
///
/// The `app` module contains helper functions for creating and running a very
/// simple Mac application environment (NSApplication) with no window or dock
/// icon.  It is provided here for the Rubrail examples to use, and may be
/// useful for simple Touch Bar-only applications, but 'real' applications
/// should probably implement the application logic themselves.
pub mod app {
    #[cfg(target_os = "macos")]
    #[cfg(feature = "private_api")]
    extern crate objc;
    extern crate log4rs;
    use std::env;
    #[cfg(not(feature = "private_api"))]
    use std::process;

    #[cfg(target_os = "macos")]
    #[cfg(feature = "private_api")]
    /// Initialize a Mac application environment (NSApplication)
    ///
    /// This is a debug/convenience function for creating a window-less Mac
    /// application.  It can be used in simple Touch Bar-only applications,
    /// and is used by the examples, but more complex applications will want
    /// to handle the application creation themselves.
    ///
    /// It initializes an NSApplication with the
    /// _NSApplicationActivationPolicyAccessory_ policy, which means it will
    /// have no window and no dock icon.
    pub fn init_app() {
        unsafe {
            let cls = objc::runtime::Class::get("NSApplication").unwrap();
            let app: *mut objc::runtime::Object = msg_send![cls, sharedApplication];
            let _ = msg_send![app, setActivationPolicy: 1]; // NSApplicationActivationPolicyAccessory
        }
    }
    #[cfg(not(feature = "private_api"))]
    ///
    pub fn init_app() {}

    #[cfg(target_os = "macos")]
    #[cfg(feature = "private_api")]
    /// Run the application's event loop forever.
    ///
    /// This is a debug/convenience function to run the Mac application's event loop
    /// forever, without returning.  It can be used with `init_app()` to run a
    /// simple application or example, but more complicated applications should
    /// implement the run loop themselves.
    pub fn run_forever() {
        unsafe {
            let cls = objc::runtime::Class::get("NSApplication").unwrap();
            let app: *mut objc::runtime::Object = msg_send![cls, sharedApplication];
            let _ = msg_send![app, run];
        }
    }
    #[cfg(not(feature = "private_api"))]
    ///
    pub fn run_forever() { loop {} }

    #[cfg(target_os = "macos")]
    #[cfg(feature = "private_api")]
    /// Terminate the application run loop and quit.
    ///
    /// This is a debug/convenience function to terminate the Mac application and
    /// end the process.  This can be used with `init_app()` and `run_forever()` for
    /// simple applications and examples, but more complex applications will want
    /// to implement custom handling for terminating.
    pub fn quit() {
        unsafe {
            let cls = objc::runtime::Class::get("NSApplication").unwrap();
            let app: *mut objc::runtime::Object = msg_send![cls, sharedApplication];
            let _ = msg_send![app, terminate: 0];
        }
    }
    #[cfg(not(feature = "private_api"))]
    ///
    pub fn quit() {
        process::exit(0);
    }

    /// Enable logging to a file in the user's home directory
    ///
    /// This is a debug function which redirects the log output of Rubrail and its
    /// examples to a text file of the given name in the user's home directory.
    ///
    /// # Arguments
    ///
    /// * `filename` - the filename **without** a path.  It is always saved in the
    /// home directory.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// extern crate rubrail;
    /// #[macro_use]
    /// extern crate log;
    /// fn main() {
    ///   rubrail::app::create_logger(".rubrail.log");
    ///   info!("This message is in ~/.rubrail.log");
    /// }
    /// ```
    pub fn create_logger(filename: &str) {
        use log::LogLevelFilter;
        use self::log4rs::append::console::ConsoleAppender;
        use self::log4rs::append::file::FileAppender;
        use self::log4rs::encode::pattern::PatternEncoder;
        use self::log4rs::config::{Appender, Config, Logger, Root};

        let log_path = format!("{}/{}", env::home_dir().unwrap().display(), filename);
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{m}{n}")))
            .build();
        let requests = FileAppender::builder()
            .build(&log_path)
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("requests", Box::new(requests)))
            .logger(Logger::builder().build("app::backend::db", LogLevelFilter::Info))
            .logger(Logger::builder()
                    .appender("requests")
                    .additive(false)
                    .build("app::requests", LogLevelFilter::Info))
            .build(Root::builder().appender("stdout").appender("requests").build(LogLevelFilter::Info))
            .unwrap();
        let _ = log4rs::init_config(config).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use Touchbar;
    use interface::TTouchbar;
    #[test]
    fn test_alloc() {
        let mut tb = Touchbar::alloc("test");
        let _ = tb.create_bar();
    }
}
