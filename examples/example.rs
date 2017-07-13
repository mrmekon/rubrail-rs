// Rubrail Example
//
// Demonstrates using Rubrail to layout and interact with a persistent Touchbar
// on Macbooks that support it.
//
// Usage notes:
//
// To access the touchbar, Rubrail *must* execute inside an OS X app bundle.
// Since Cargo doesn't do that itself, this example is packaged with a second
// wrapper example (example_launcher), and a bundling script (example.sh).
// They can all be used together to setup and run the example from a bundle.
//
// Simply run:
//
// $ cargo test && cargo run --example example_launcher
//
//
extern crate rubrail;

use rubrail::Touchbar;
use rubrail::TTouchbar;
use rubrail::TScrubberData;
use rubrail::ItemId;

use std::rc::Rc;
use std::cell::RefCell;

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
        let width = len * 8 + 20;
        width
    }
    fn touch(&self, _item: ItemId, idx: u32) {
        println!("scrub touch: {}", idx);
    }
}

fn populate(bar_rc: Rc<RefCell<Touchbar>>, count: u32) {
    // Get touchbar from the refcell.  It's wrapped in a cell so
    // it can be passed around in the button callbacks.
    let mut tb = (bar_rc).borrow_mut();

    // Create the lowest level "root" touchbar
    let barid = tb.create_bar();

    // Create a quit button for root bar
    let quit_id = tb.create_button(None, Some("Quit"), Box::new(move |_| {rubrail::quit()}));

    // Create an action button for the root bar.  When clicked, it will
    // close the bar and re-create itself.
    let bar_copy = bar_rc.clone();
    let text = format!("button{}", count);
    let button1_id = tb.create_button(None, Some(&text), Box::new(move |_| {
        populate(bar_copy.clone(), count+1)
    }));

    // Create a text label for the root bar
    let label1_id = tb.create_label("This is a label\nWith two rows");

    // Create a data backend for scrolling text "scrubbers"
    let scrubber = Rc::new(TouchbarHandler {
        devices: RefCell::new(vec![
            "one".to_string(), "two".to_string(),
            "a little bit longer one".to_string(),
            "three".to_string(),
            "this one is really quite a bit longer than the others".to_string()]),
    });

    // Create a scrubber for the root bar
    let scrubber1_id = tb.create_text_scrubber(scrubber.clone());
    tb.select_scrubber_item(scrubber1_id, 1);

    // Create a 'popbar', a second level deep bar
    let popbar1_id = tb.create_bar();
    let popbutton1_id = tb.create_popover_item(None, Some("Popbar1"), popbar1_id);

    // Create another scrubber with the same data, for the popbar.
    // Note that the data and callbacks are shared, but this is a different
    // instance and can store a different active selection.
    let scrubber2_id = tb.create_text_scrubber(scrubber.clone());
    tb.select_scrubber_item(scrubber2_id, 3);

    // Create a slider for the popbar.
    let slider1_id = tb.create_slider(0.0, 50.0, Box::new(move |_s,v| {println!("Slid to: {}", v);}));
    tb.update_slider(slider1_id, 15.0);

    // Create a another popbar.  This will make a 2-level deep UI.
    let popbar2_id = tb.create_bar();
    let popbutton2_id = tb.create_popover_item(None, Some("Popbar2"), popbar2_id);

    // Create buttons to display on the popbars
    let popbar_button_id = tb.create_button(None, Some("1 level deep"), Box::new(move |_| {}));
    let deep_button_id = tb.create_button(None, Some("2 levels deep"), Box::new(move |_| {}));

    // Layout the deepest (2-level) popbar
    tb.add_items_to_bar(popbar2_id, vec![deep_button_id]);

    // Layout the middle (1-level) popbar
    tb.add_items_to_bar(popbar1_id, vec![popbar_button_id, popbutton2_id, slider1_id, scrubber2_id]);

    // Layout the root bar
    tb.add_items_to_bar(barid, vec![quit_id, button1_id, popbutton1_id, label1_id, scrubber1_id]);

    // Register the root bar and display it.
    tb.set_bar_as_root(barid, true);
    tb.enable();
}

fn main() {
    // Initialize OS X application.  A real app should probably not use this.
    rubrail::init_app();

    // Initialize the touchbar
    let bar_rc = Rc::new(RefCell::new(Touchbar::alloc("bar")));

    // Populate the touchbar with UI elements
    populate(bar_rc.clone(), 1);

    // Enter OS X application loop.  A real application should probably implement
    // this itself.
    rubrail::run_forever();
}
