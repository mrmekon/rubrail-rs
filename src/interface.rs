use std::rc::Rc;

pub type BarId = u64;
pub type ItemId = u64;
pub type ButtonCb = Box<Fn(u64)>;
pub type SliderCb = Box<Fn(u64, f64)>;

pub trait TScrubberData {
    fn count(&self, item: ItemId) -> u32;
    fn text(&self, item: ItemId, idx: u32) -> String;
    fn width(&self, item: ItemId, idx: u32) -> u32;
    fn touch(&self, item: ItemId, idx: u32);
}

pub trait TTouchbar {
    type T: TTouchbar;
    fn alloc(title: &str) -> Self::T;
    fn set_icon(&self, image: &str) {}
    fn enable(&self) {}

    fn create_bar(&mut self) -> BarId { 0 }
    fn add_items_to_bar(&mut self, bar_id: &BarId, items: Vec<ItemId>) {}
    fn set_bar_as_root(&mut self, bar_id: &BarId) {}

    fn create_popover_item(&mut self, image: Option<&str>,
                           text: Option<&str>, bar_id: &BarId) -> ItemId {0}

    fn create_label(&mut self, text: &str) -> ItemId {0}
    fn update_label(&mut self, label_id: &ItemId, text: &str) {}

    fn create_text_scrubber(&mut self, data: Rc<TScrubberData>) -> ItemId {0}
    fn select_scrubber_item(&mut self, scrub_id: &ItemId, index: u32) {}
    fn refresh_scrubber(&mut self, scrub_id: &ItemId) {}

    fn create_button(&mut self, image: Option<&str>, text: Option<&str>, cb: ButtonCb) -> ItemId {0}

    fn create_slider(&mut self, min: f64, max: f64, cb: SliderCb) -> ItemId {0}
    fn update_slider(&mut self, id: &ItemId, value: f64) {}
}
