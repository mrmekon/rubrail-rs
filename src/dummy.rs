#[allow(dead_code)]
use super::interface::*;

///
pub struct DummyTouchbar {}

#[allow(dead_code)]
impl TTouchbar for DummyTouchbar {
    type T = DummyTouchbar;
    fn alloc(_title: &str) -> DummyTouchbar { DummyTouchbar {} }
}

///
pub mod util {
    use super::ItemId;
    ///
    pub fn print_nsstring(_str: *mut u64) {}
    ///
    pub fn nsstring_decode(_str: *mut u64) -> String { String::new() }
    ///
    pub fn bundled_resource_path(_name: &str, _extension: &str) -> Option<String> { None }
    ///
    pub unsafe fn set_bg_color(_item: &ItemId, _r: f64, _g: f64, _b: f64, _alpha: f64) { }
    ///
    pub unsafe fn set_text_color(_item: &ItemId, _r: f64, _g: f64, _b: f64, _alpha: f64) { }
}
