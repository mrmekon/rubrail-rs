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
//! The included example uses [fruitbasket](https://github.com/mrmekon/fruitbasket)
//! to automatically bundle itself into an OS X app at runtime.  You can run the
//! example with:
//!
//! `$ cargo test && cargo run --example example`
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

#[allow(unused_imports)]
#[macro_use]
extern crate log;

//
// Mac+TouchBar imports
//
#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
#[macro_use]
mod wrapper;

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

#[cfg(target_os = "macos")]
#[cfg(feature = "private_api")]
pub use touchbar::util;


//
// Non-Mac/Dummy TouchBar imports
//
#[cfg(not(all(target_os = "macos", feature = "private_api")))]
mod dummy;

#[cfg(not(all(target_os = "macos", feature = "private_api")))]
pub use dummy::DummyTouchbar as Touchbar;

#[cfg(not(all(target_os = "macos", feature = "private_api")))]
pub use dummy::util;

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
