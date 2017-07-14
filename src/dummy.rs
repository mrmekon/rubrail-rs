#[allow(dead_code)]
use super::interface::*;

pub struct DummyTouchbar {}

#[allow(dead_code)]
impl TTouchbar for DummyTouchbar {
    type T = DummyTouchbar;
    fn alloc(_title: &str) -> DummyTouchbar { DummyTouchbar {} }
}
