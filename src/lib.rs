#[allow(dead_code)]
#[allow(unused_variables)]
mod interface;
pub use interface::*;

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
        let cls = objc::runtime::Class::get("NSAutoreleasePool").unwrap();
        let pool: *mut objc::runtime::Object = msg_send![cls, alloc];
        let pool: *mut objc::runtime::Object = msg_send![pool, init];
        let _ = pool;
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

pub fn quit() {
    unsafe {
        let cls = objc::runtime::Class::get("NSApplication").unwrap();
        let app: *mut objc::runtime::Object = msg_send![cls, sharedApplication];
        let _ = msg_send![app, terminate: 0];
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
        tb.enable();
    }
}
