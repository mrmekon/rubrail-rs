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
extern crate log4rs;
use std::env;

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
#[macro_use]
extern crate objc;

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
#[macro_use]
mod touchbar;

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
pub use touchbar::Touchbar as Touchbar;

#[cfg(not(feature = "private_api"))]
mod dummy;

#[cfg(not(feature = "private_api"))]
pub use dummy::DummyTouchbar as Touchbar;

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
pub fn init_app() {
    unsafe {
        let cls = objc::runtime::Class::get("NSApplication").unwrap();
        let app: *mut objc::runtime::Object = msg_send![cls, sharedApplication];
        let _ = msg_send![app, setActivationPolicy: 1]; // NSApplicationActivationPolicyAccessory
    }
}
#[cfg(not(feature = "private_api"))]
pub fn init_app() {}

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
pub fn run_forever() {
    unsafe {
        let cls = objc::runtime::Class::get("NSApplication").unwrap();
        let app: *mut objc::runtime::Object = msg_send![cls, sharedApplication];
        let _ = msg_send![app, run];
    }
}
#[cfg(not(feature = "private_api"))]
pub fn run_forever() { loop {} }

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
pub fn quit() {
    unsafe {
        let cls = objc::runtime::Class::get("NSApplication").unwrap();
        let app: *mut objc::runtime::Object = msg_send![cls, sharedApplication];
        let _ = msg_send![app, terminate: 0];
    }
}
#[cfg(not(feature = "private_api"))]
pub fn quit() {
    std::process::exit(0);
}

pub fn create_logger(filename: &str) {
    use log::LogLevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::append::file::FileAppender;
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::config::{Appender, Config, Logger, Root};

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

#[cfg(test)]
mod tests {
    use Touchbar;
    use interface::TTouchbar;
    #[test]
    fn test_alloc() {
        let mut tb = Touchbar::alloc("test");
        let _ = tb.create_bar();
        tb.enable();
    }
}
