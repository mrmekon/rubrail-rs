# Rubrail
[![Build Status](https://api.travis-ci.org/mrmekon/rubrail-rs.svg?branch=master)](https://travis-ci.org/mrmekon/rubrail-rs)

Rubrail is a Rust library for building Touch Bar interfaces for Mac OS X.

It creates a persistent icon in the 'Control Strip' region on the right side of the touchbar.  This lets you access a Touch Bar menu of your design at any time, regardless of which application is currently active.

Using a pure-Rust API, you can create Touch Bar UIs with the most common items: Buttons, Labels, Sliders, Scrubbers (horizontally scrolling text selections), and recursive menus.

Source for following screencast in `examples/example.rs`
<img src="https://github.com/mrmekon/rubrail-rs/blob/master/docs/screencast.gif" width="1200">

## Run the example

`$ cargo test && cargo run --example example_launcher`

## Cargo Crate

[![Crates.io Version](https://img.shields.io/crates/v/rubrail.svg)](https://crates.io/crates/rubrail)

## Documentation

[Rubrail API documentation](https://mrmekon.github.io/rubrail-rs/rubrail/)

## Information

### Warning -- Private APIs

Note that access to the Control Strip is forbidden by Apple's guidelines.  Rubrail uses *private APIs* to create its menus, and thus is not suitable for distribution through the App Store.  A 'dummy' implementation is provided for apps that want to provide Touch Bar support, but want the ability to avoid linking against private frameworks when distributing.  Build with the `--no-default-features` Cargo flag to get a dummy implementation that does nothing.

### Note -- App bundle required

To communicate with the Touch Bar service, apps using Rubrail *must* be executed from an app bundle (.app).  Running the executable directly will not crash, but the icon will not be registered with the Touch Bar service, so your Touch Bar UI will be unavailable.

The included example comes with a bundling script (examples/example.sh) and a launcher (examples/example_launcher.rs) to move itself into an app bundle and execute.


### Limitations

There is no support for changing the UI of an existing bar.  To change the UI layout, you must create a new bar and completely replace the old one.  Scrubbers are an exception: their contents are managed by callbacks, but they do not live-refresh when the bar is visible.  The user must close and re-open the bar to see scrubber content changes.

The Touch Bar API supports doing just about anything with custom views.  Rubrail does not.  Only a very limited set of UI options are exposed.

#### Linking

Rubrail links against the private Apple framework `DFRFoundation.framework`.  This is handled by adding the private framework path in `build.rs`.  Applications that _use_ Rubrail do not need to link against it themselves, but should be aware that a dependency does.

Linking requires an XCode version high enough to support the Touch Bar and the private framework.  The only tested version of XCode is version 8.3.  Applications that use Rubrail **must** build with a new enough XCode version.  If using Travis-CI, this means adding a line like this to your `.travis.yml`:

`osx_image: xcode8.3`


### Known Bugs

Memory leaks!  Memory leaks as far as the eye can see.
