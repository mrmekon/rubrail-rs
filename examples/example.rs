// Rubrail Example
//
// Demonstrates using Rubrail to layout and interact with a persistent Touchbar
// on Macbooks that support it.
//
// Usage notes:
//
// To access the touchbar, Rubrail *must* execute inside an OS X app bundle.
// Since Cargo doesn't do that itself, this example uses the Trampoline feature
// of the fruitbasket crate to relaunch itself in an app bundle.
//
// Simply run:
//
// $ cargo test && cargo run --example example
//
//
extern crate fruitbasket;
extern crate rubrail;

use rubrail::ItemId;
use rubrail::SwipeState;
use rubrail::TScrubberData;
use rubrail::TTouchbar;
use rubrail::Touchbar;

#[macro_use]
extern crate log;

use std::cell::RefCell;
use std::rc::Rc;

struct TouchbarHandler {
    devices: RefCell<Vec<String>>,
}
impl TScrubberData for TouchbarHandler {
    fn count(&self, _item: ItemId) -> u32 {
        self.devices.borrow().len() as u32
    }
    fn text(&self, _item: ItemId, idx: u32) -> String {
        self.devices.borrow()[idx as usize].to_string()
    }
    fn width(&self, _item: ItemId, idx: u32) -> u32 {
        // 10px per character + some padding seems to work nicely for the default
        // font.  no idea what it's like on other machines.  does the touchbar
        // font change? ¯\_(ツ)_/¯
        let len = self.devices.borrow()[idx as usize].len() as u32;

        len * 8 + 20
    }
    fn touch(&self, _item: ItemId, idx: u32) {
        info!("scrub touch: {}", idx);
    }
}

fn populate(bar_rc: Rc<RefCell<Touchbar>>, count: u32, stopper: fruitbasket::FruitStopper) {
    // Get touchbar from the refcell.  It's wrapped in a cell so
    // it can be passed around in the button callbacks.
    let mut tb = (bar_rc).borrow_mut();

    // Create the lowest level "root" touchbar
    let barid = tb.create_bar();

    // Create a quit button for root bar
    let quit_stopper = stopper.clone();
    let quit_id = tb.create_button(
        None,
        Some("Quit"),
        Box::new(move |_| {
            quit_stopper.stop();
        }),
    );

    // Create an action button for the root bar.  When clicked, it will
    // close the bar and re-create itself.
    let bar_copy = bar_rc.clone();
    let text = format!("button{}", count);
    let button1_id = tb.create_button(
        None,
        Some(&text),
        Box::new(move |_| populate(bar_copy.clone(), count + 1, stopper.clone())),
    );

    // Create a text label for the root bar
    let label1_id = tb.create_label("This is a label");
    tb.update_label(&label1_id, "This is a label\nWith two rows");
    tb.update_label_width(&label1_id, 100);

    // Support double-clicking the label with one finger
    tb.add_item_tap_gesture(
        &label1_id,
        2,
        1,
        Box::new(move |_item| {
            info!("Label double-clicked!");
        }),
    );

    // Add a swipe gesture to the label that changes the text color to
    // increasingly green as you swipe right, or increasingly red as you swipe
    // left, and resets to white when released.
    tb.add_item_swipe_gesture(
        &label1_id,
        Box::new(move |item, state, translation| {
            let color: f64 = match translation.abs().trunc() as u32 {
                t if t < 10 => 1.0,
                t if t > 100 => 0.0,
                _ => (45. / translation.abs()),
            };

            let rgba = if let SwipeState::Ended = state {
                (1.0, 1.0, 1.0, 1.0)
            } else if translation.is_sign_positive() {
                (color, 1.0, color, 1.0)
            } else {
                (1.0, color, color, 1.0)
            };

            unsafe {
                rubrail::util::set_text_color(item, rgba.0, rgba.1, rgba.2, rgba.3);
            }
        }),
    );

    // Create a data backend for scrolling text "scrubbers"
    let scrubber = Rc::new(TouchbarHandler {
        devices: RefCell::new(vec![
            "one".to_string(),
            "two".to_string(),
            "a little bit longer one".to_string(),
            "three".to_string(),
            "this one is really quite a bit longer than the others".to_string(),
        ]),
    });

    // Create a scrubber for the root bar
    let scrubber1_id = tb.create_text_scrubber(scrubber.clone());
    tb.select_scrubber_item(&scrubber1_id, 1);

    // Create a 'popbar', a second level deep bar
    let popbar1_id = tb.create_bar();
    let popbutton1_id = tb.create_popover_item(None, Some("Popbar1"), &popbar1_id);

    // Create another scrubber with the same data, for the popbar.
    // Note that the data and callbacks are shared, but this is a different
    // instance and can store a different active selection.
    let scrubber2_id = tb.create_text_scrubber(scrubber.clone());
    tb.select_scrubber_item(&scrubber2_id, 3);

    // Create a slider for the popbar.
    let slider1_id = tb.create_slider(
        0.0,
        50.0,
        Some("Slide"),
        true,
        Box::new(move |_s, v| {
            info!("Slid to: {}", v);
        }),
    );
    tb.update_slider(&slider1_id, 15.0);

    // Create a another popbar.  This will make a 2-level deep UI.
    let popbar2_id = tb.create_bar();
    let popbutton2_id = tb.create_popover_item(None, Some("Popbar2"), &popbar2_id);

    // Create buttons to display on the popbars
    let popbar_button_id = tb.create_button(None, Some("1 level deep"), Box::new(move |_| {}));
    let deep_button_id = tb.create_button(None, Some("2 levels deep"), Box::new(move |_| {}));

    // Layout the deepest (2-level) popbar
    tb.add_items_to_bar(&popbar2_id, vec![deep_button_id]);

    // Layout the middle (1-level) popbar
    tb.add_items_to_bar(
        &popbar1_id,
        vec![popbar_button_id, popbutton2_id, slider1_id, scrubber2_id],
    );

    // Layout the root bar
    tb.add_items_to_bar(
        &barid,
        vec![quit_id, button1_id, popbutton1_id, label1_id, scrubber1_id],
    );

    // Register the root bar and display it.
    tb.set_bar_as_root(barid);
}

fn main() -> Result<(), ()> {
    // Write log to home directory
    fruitbasket::create_logger(".rubrail.log", fruitbasket::LogDir::Home, 5, 2).unwrap();

    // Initialize OS X application.
    let icon = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("icon.png");
    let mut nsapp = fruitbasket::Trampoline::new(
        "rubrail_example",
        "rubrail_example",
        "com.trevorbentley.rubrail_example",
    )
    .icon("icon.png")
    .version(env!("CARGO_PKG_VERSION"))
    .plist_key("LSBackgroundOnly", "1")
    .resource(icon.to_str().unwrap())
    .build(fruitbasket::InstallDir::Custom("target/".to_string()))
    .unwrap();
    nsapp.set_activation_policy(fruitbasket::ActivationPolicy::Prohibited);

    // Initialize the touchbar
    let bar_rc = Rc::new(RefCell::new(Touchbar::alloc("bar")));

    let stopper = nsapp.stopper();
    // Populate the touchbar with UI elements
    populate(bar_rc.clone(), 1, stopper);

    // Enter OS X application loop.
    nsapp.run(fruitbasket::RunPeriod::Forever)
}
