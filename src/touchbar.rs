extern crate objc;
extern crate objc_foundation;
extern crate objc_id;
extern crate cocoa;

use super::interface::*;

use std::fmt;
use std::rc::Rc;
use std::cell::Cell;
use std::sync::{Once, ONCE_INIT};
use std::collections::BTreeMap;

use objc::Message;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use self::objc_foundation::{INSObject, NSObject};
use self::cocoa::base::{nil, YES, NO, SEL};
use self::cocoa::foundation::NSString;
use self::cocoa::foundation::{NSRect, NSPoint, NSSize};
use self::cocoa::appkit::{NSApp, NSImage};
use self::objc_id::Id;
use self::objc_id::Shared;

use super::wrapper::RRTouchBar;
use super::wrapper::RRCustomTouchBarItem;
use super::wrapper::RRScrubber;
use super::wrapper::RRPopoverTouchBarItem;
use super::wrapper::RRSliderTouchBarItem;

const IDENT_PREFIX: &'static str = "com.trevorbentley.";

/// Controller for creating and using Touch Bar UIs
///
/// The `Touchbar` type provides the interface between Rust and the Apple Touch
/// Bar system.  It handles communication with the Touch Bar service, delegation
/// of UI and touch events, and memory management of Objective-C objects.  All
/// creation and modification of Touch Bar UIs is done through it.
///
/// This implementation uses **private** Apple APIs to create an always-available
/// menu to the Control Strip region, which can be accessed regardless of which
/// application is in focus.  Apps using it are **not permitted in the App Store**.
///
/// There should typically be **one** `Touchbar` type alocated per application.
/// Rubrail does not enforce the singleton pattern automatically, so it is up to
/// the application to guarantee that there is only one.  Theoretically, one
/// application _can_ have multiple disjoint entries in the Touch Bar by creating
/// more than one `Touchbar`, and Rubrail allows this though it is not tested.
///
/// See the documentation of the [`TTouchbar`](trait.TTouchbar.html) trait for usage.
///
/// # Example
///
/// ```
/// extern crate rubrail;
/// use rubrail::TTouchbar;
/// fn main() {
///   let controller = rubrail::Touchbar::alloc("test");
/// }
/// ```
pub type Touchbar = Box<RustTouchbarDelegateWrapper>;

pub type Ident = u64;

#[link(name = "DFRFoundation", kind = "framework")]
extern {
    pub fn DFRSystemModalShowsCloseBoxWhenFrontMost(x: i8);
    pub fn DFRElementSetControlStripPresenceForIdentifier(n: *mut Object, x: i8);
}

pub mod util {
    //! Utility functions for working with the Apple/TouchBar environment
    //!
    //! Contains convenience functions that help with some common actions in
    //! Mac environments.

    extern crate libc;
    extern crate cocoa;
    use super::ItemId;
    use std::ptr;
    use std::ffi::CStr;
    use objc::runtime::Object;
    use objc::runtime::Class;
    use self::cocoa::foundation::NSString;
    use self::cocoa::base::nil;
    #[allow(dead_code)]
    /// Print an NSString object to the global logger
    pub fn print_nsstring(str: *mut Object) {
        unsafe {
            let cstr: *const libc::c_char = msg_send![str, UTF8String];
            let rstr = CStr::from_ptr(cstr).to_string_lossy().into_owned();
            info!("{}", rstr);
        }
    }

    /// Convert an NSString object into a Rust String
    pub fn nsstring_decode(str: *mut Object) -> String {
        unsafe {
            let cstr: *const libc::c_char = msg_send![str, UTF8String];
            let rstr = CStr::from_ptr(cstr).to_string_lossy().into_owned();
            rstr
        }
    }

    /// Locate a resource in the executing Mac App bundle
    ///
    /// If the program is executing from an App bundle (.app), which it must be
    /// to use Rubrail correctly, this looks for a resource by name and
    /// extension in the bundled Resources directory.
    pub fn bundled_resource_path(name: &str, extension: &str) -> Option<String> {
        unsafe {
            let cls = Class::get("NSBundle").unwrap();
            let bundle: *mut Object = msg_send![cls, mainBundle];
            let res = NSString::alloc(nil).init_str(name);
            let ext = NSString::alloc(nil).init_str(extension);
            let ini: *mut Object = msg_send![bundle, pathForResource:res ofType:ext];
            let _ = msg_send![res, release];
            let _ = msg_send![ext, release];
            let cstr: *const libc::c_char = msg_send![ini, UTF8String];
            if cstr != ptr::null() {
                let rstr = CStr::from_ptr(cstr).to_string_lossy().into_owned();
                return Some(rstr);
            }
            None
        }
    }

    /// Sets the backgroundColor attribute on an item's view.
    ///
    /// This is an **unsafe** helper function to set the backgroundColor
    /// attribute on the view of an item.  It does *not* verify that the item
    /// actually has a view, nor that the view supports backgroundColor, hence
    /// **unsafe**.
    ///
    /// Known compatible items: labels
    ///
    /// # Arguments
    ///
    /// * `item` - The `ItemId` to color
    /// * `r` - Red value (0.0 - 1.0)
    /// * `g` - Green value (0.0 - 1.0)
    /// * `b` - Blue value (0.0 - 1.0)
    /// * `alpha` - Alpha value (0.0 - 1.0)
    pub unsafe fn set_bg_color(item: &ItemId, r: f64, g: f64, b: f64, alpha: f64) {
        let item = *item as *mut Object;
        let view: *mut Object = msg_send![item, view];
        let cls = Class::get("NSColor").unwrap();
        let color: *mut Object = msg_send![
            cls, colorWithRed: r green: g blue: b alpha: alpha];
        msg_send![view, setBackgroundColor: color];
    }

    /// Sets the textColor attribute on an item's view.
    ///
    /// This is an **unsafe** helper function to set the textColor attribute on
    /// the view of an item.  It does *not* verify that the item actually has a
    /// view, nor that the view supports textColor, hence **unsafe**.
    ///
    /// Known compatible items: labels
    ///
    /// # Arguments
    ///
    /// * `item` - The `ItemId` to color
    /// * `r` - Red value (0.0 - 1.0)
    /// * `g` - Green value (0.0 - 1.0)
    /// * `b` - Blue value (0.0 - 1.0)
    /// * `alpha` - Alpha value (0.0 - 1.0)
    pub unsafe fn set_text_color(item: &ItemId, r: f64, g: f64, b: f64, alpha: f64) {
        let item = *item as *mut Object;
        let view: *mut Object = msg_send![item, view];
        let cls = Class::get("NSColor").unwrap();
        let color: *mut Object = msg_send![
            cls, colorWithRed: r green: g blue: b alpha: alpha];
        msg_send![view, setTextColor: color];
    }
}

macro_rules! image_template {
    ( $var:ident, $($template:ident),* ) => {
        match $var {
            $(
                ImageTemplate::$template => format!("NSTouchBar{}", stringify!($template)),
            )*
        }
    }
}

impl ImageTemplate {
    fn objc(template: ImageTemplate) -> *mut Object {
        unsafe {
            let s = image_template!(
                template,
                AddDetailTemplate,
                AddTemplate,
                AlarmTemplate,
                AudioInputMuteTemplate,
                AudioInputTemplate,
                AudioOutputMuteTemplate,
                AudioOutputVolumeHighTemplate,
                AudioOutputVolumeLowTemplate,
                AudioOutputVolumeMediumTemplate,
                AudioOutputVolumeOffTemplate,
                BookmarksTemplate,
                ColorPickerFill,
                ColorPickerFont,
                ColorPickerStroke,
                CommunicationAudioTemplate,
                CommunicationVideoTemplate,
                ComposeTemplate,
                DeleteTemplate,
                DownloadTemplate,
                EnterFullScreenTemplate,
                ExitFullScreenTemplate,
                FastForwardTemplate,
                FolderCopyToTemplate,
                FolderMoveToTemplate,
                FolderTemplate,
                GetInfoTemplate,
                GoBackTemplate,
                GoDownTemplate,
                GoForwardTemplate,
                GoUpTemplate,
                HistoryTemplate,
                IconViewTemplate,
                ListViewTemplate,
                MailTemplate,
                NewFolderTemplate,
                NewMessageTemplate,
                OpenInBrowserTemplate,
                PauseTemplate,
                PlayheadTemplate,
                PlayPauseTemplate,
                PlayTemplate,
                QuickLookTemplate,
                RecordStartTemplate,
                RecordStopTemplate,
                RefreshTemplate,
                RewindTemplate,
                RotateLeftTemplate,
                RotateRightTemplate,
                SearchTemplate,
                ShareTemplate,
                SidebarTemplate,
                SkipAhead15SecondsTemplate,
                SkipAhead30SecondsTemplate,
                SkipAheadTemplate,
                SkipBack15SecondsTemplate,
                SkipBack30SecondsTemplate,
                SkipBackTemplate,
                SkipToEndTemplate,
                SkipToStartTemplate,
                SlideshowTemplate,
                TagIconTemplate,
                TextBoldTemplate,
                TextBoxTemplate,
                TextCenterAlignTemplate,
                TextItalicTemplate,
                TextJustifiedAlignTemplate,
                TextLeftAlignTemplate,
                TextListTemplate,
                TextRightAlignTemplate,
                TextStrikethroughTemplate,
                TextUnderlineTemplate,
                UserAddTemplate,
                UserGroupTemplate,
                UserTemplate
            );
            let name = NSString::alloc(nil).init_str(&s);
            let _ = msg_send![name, autorelease];
            name
        }
    }
}

#[derive(PartialEq, Debug)]
enum ItemType {
    Button,
    Label,
    Slider,
    Scrubber,
    Popover,
    Spacer,
}

struct InternalBar {
    view: *mut Object,
    ident: Ident,
    items: Vec<ItemId>,
}

impl fmt::Display for InternalBar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bar[{}] ({}) <{:x}>",
               self.items.len(),
               util::nsstring_decode(self.ident as *mut Object),
               self.view as u64)
    }
}

struct InternalItem {
    _type: ItemType,
    view: *mut Object,
    ident: Ident,
    control: Option<*mut Object>,
    scrubber: Option<Rc<TScrubberData>>,
    button_cb: Option<ButtonCb>,
    slider_cb: Option<SliderCb>,
    swipe_cb: Option<SwipeCb>,
    tap_cb: Option<ButtonCb>,
    child_bar: Option<ItemId>,
}

impl fmt::Display for InternalItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} ({}) <{:x}>", self._type,
               util::nsstring_decode(self.ident as *mut Object),
               self.view as u64)
    }
}

impl InternalItem {
    fn free_objc_allocations(&mut self) {
        unsafe {
            if let Some(obj) = self.control {
                // Sliders don't allocate their control
                if self._type != ItemType::Slider {
                    let _ = msg_send![obj, release];
                }
            }
            let _ = msg_send![self.view, release];
            let ident = self.ident as *mut Object;
            let _ = msg_send![ident, release];
            self.view = nil;
            self.ident = 0;
            self.control = None;
            self.scrubber = None;
            self.button_cb = None;
            self.swipe_cb = None;
            self.tap_cb = None;
            self.slider_cb = None;
        }
    }
}

pub struct RustTouchbarDelegateWrapper {
    objc: Id<ObjcAppDelegate, Shared>,
    next_item_id: Cell<u64>,
    bar_map: BTreeMap<ItemId, InternalBar>,
    item_map: BTreeMap<ItemId, InternalItem>,
}

impl RustTouchbarDelegateWrapper {
    fn generate_ident(&mut self) -> u64 {
        unsafe {
            // Create string identifier
            let next_item_id = self.next_item_id.get();
            self.next_item_id.set(next_item_id + 1);
            let ident = format!("{}{}", IDENT_PREFIX, next_item_id);
            let objc_ident = NSString::alloc(nil).init_str(&ident);
            objc_ident as u64
        }
    }
    fn alloc_button(&mut self, image: Option<&TouchbarImage>, text: Option<&str>,
                    target: *mut Object, sel: SEL) -> *mut Object {
        unsafe {
            let text = match text {
                Some(s) => NSString::alloc(nil).init_str(s),
                None => nil,
            };
            let image = match image {
                Some(i) => *i as *mut Object,
                None => nil,
            };
            let cls = Class::get("NSButton").unwrap();
            let btn: *mut Object;
            // Match on (image, text) as booleans.   false == null.
            match ((image as u64) != 0, (text as u64) != 0) {
                (false,true) => {
                    btn = msg_send![cls,
                                    buttonWithTitle: text
                                    target:target
                                    action:sel];
                }
                (true,false) => {
                    btn = msg_send![cls,
                                    buttonWithImage: image
                                    target:target
                                    action:sel];
                }
                (true,true) => {
                    btn = msg_send![cls,
                                    buttonWithTitle: text
                                    image:image
                                    target:target
                                    action:sel];
                }
                _ => { return nil }
            }
            if text != nil {
                let _ = msg_send![text, release];
            }
            if image != nil {
                let _ = msg_send![image, release];
            }
            // These are auto-released, so they need an explicit retain
            let _ = msg_send![btn, retain];
            btn
        }
    }
    fn find_view(&self, ident: Ident) -> Option<*mut Object> {
        match self.item_map.values().into_iter().filter(|x| {
            unsafe {
                let id = ident as *mut Object;
                let equal: bool = msg_send![id, isEqualToString: x.ident];
                equal
            }
        }).next() {
            Some(item) => Some(item.view),
            None => None,
        }
    }
    fn find_view_from_control(&self, item: &ItemId) -> Option<*mut Object> {
        match self.item_map.values().into_iter().filter(|x| {
            x.control.is_some() && x.control.unwrap() as ItemId == *item
        }).next() {
            Some(item) => Some(item.view),
            None => None,
        }
    }
    fn find_bar_ident(&self, bar: &ItemId) -> Option<Ident> {
        match self.bar_map.values().into_iter().filter(|x| {
            x.view as ItemId == *bar
        }).next() {
            Some(item) => Some(item.ident),
            None => None,
        }
    }
    fn find_ident(&self, item: &ItemId) -> Option<Ident> {
        match self.item_map.values().into_iter().filter(|x| {
            x.view as ItemId == *item
        }).next() {
            Some(item) => Some(item.ident),
            None => None,
        }
    }
    fn find_ident_from_control(&self, item: &ItemId) -> Option<Ident> {
        match self.item_map.values().into_iter().filter(|x| {
            x.control.is_some() && x.control.unwrap() as ItemId == *item
        }).next() {
            Some(item) => Some(item.ident),
            None => None,
        }
    }
    fn find_button_cb(&self, btn: u64) -> Option<&ButtonCb> {
        match self.item_map.values().into_iter().filter(|x| {
            x._type == ItemType::Button && x.control.unwrap() as u64 == btn
        }).next() {
            Some(item) => item.button_cb.as_ref(),
            None => None,
        }
    }
    fn find_swipe_cb(&self, item: u64) -> Option<&SwipeCb> {
        match self.item_map.values().into_iter().filter(|x| {
            x.control.is_some() && x.control.unwrap() as u64 == item
        }).next() {
            Some(item) => item.swipe_cb.as_ref(),
            None => None,
        }
    }
    fn find_tap_cb(&self, item: u64) -> Option<&ButtonCb> {
        match self.item_map.values().into_iter().filter(|x| {
            x.control.is_some() && x.control.unwrap() as u64 == item
        }).next() {
            Some(item) => item.tap_cb.as_ref(),
            None => None,
        }
    }
    fn find_slider_cb(&self, sldr: u64) -> Option<&SliderCb> {
        match self.item_map.values().into_iter().filter(|x| {
            x._type == ItemType::Slider && x.view as u64 == sldr
        }).next() {
            Some(item) => item.slider_cb.as_ref(),
            None => None,
        }
    }
    fn find_scrubber(&self, scrubber: u64) -> Option<ItemId> {
        match self.item_map.values().into_iter().filter(|x| {
            x._type == ItemType::Scrubber && x.control.unwrap() as u64 == scrubber
        }).next() {
            Some(item) => Some(item.view as ItemId),
            None => None,
        }
    }
    fn find_scrubber_callbacks(&self, scrubber: u64) -> Option<&Rc<TScrubberData>> {
        match self.item_map.values().into_iter().filter(|x| {
            x._type == ItemType::Scrubber && x.control.unwrap() as u64 == scrubber
        }).next() {
            Some(item) => item.scrubber.as_ref(),
            None => None,
        }
    }
    fn find_popover(&self, button: u64) -> Option<ItemId> {
        match self.item_map.values().into_iter().filter(|x| {
            x._type == ItemType::Popover && x.control.unwrap() as u64 == button
        }).next() {
            Some(item) => Some(item.view as ItemId),
            None => None,
        }
    }
    fn free_bar_allocations(&mut self, bar: *mut Object) {
        let bar_id = bar as u64;
        let mut subbars = Vec::<*mut Object>::new();
        let items = self.bar_map.get(&bar_id).unwrap().items.clone();
        for item in items.iter() {
            let mut internal_item = self.item_map.remove(&item).unwrap();
            if internal_item._type == ItemType::Popover {
                subbars.push(internal_item.child_bar.unwrap() as *mut Object);
            }
            internal_item.free_objc_allocations();
        }
        self.bar_map.get_mut(&bar_id).unwrap().items.clear();
        for subbar in subbars {
            self.free_bar_allocations(subbar);
        }
    }
    unsafe fn set_label_font_for_text(label: *mut Object, text: &str) {
        //let constraints: *mut Object = msg_send![label, constraints];
        //let height_constraint: *mut Object = msg_send![constraints, firstObject];
        //if height_constraint != nil {
        //    // TODO: if re-enabled, must use NSLayoutConstraint identifier to
        //    // make sure the correct constraint is removed
        //    msg_send![label, removeConstraint: height_constraint];
        //}
        if text.contains("\n") {
            // Shrink font for multi-line labels.  This makes for quite small text.
            let cls = Class::get("NSFont").unwrap();
            let default_size:f64 = msg_send![cls, systemFontSize];
            let custom_font: *mut Object = msg_send![cls, systemFontOfSize: default_size - 3.0];
            msg_send![label, setFont: custom_font];

            //let anchor: *mut Object = msg_send![label, heightAnchor];
            //let constraint: *mut Object = msg_send![anchor, constraintEqualToConstant: 30. as f64];
            //let _ = msg_send![constraint, setActive: YES];
        }
        else {
            // Enlarge the font for single lines.  For some reason it defaults
            // to smaller than other elements.
            let cls = Class::get("NSFont").unwrap();
            let default_size:f64 = msg_send![cls, systemFontSize];
            let custom_font: *mut Object = msg_send![cls, systemFontOfSize: default_size + 3.0];
            msg_send![label, setFont: custom_font];
        }
    }
}

impl TTouchbar for Touchbar {
    type T = Touchbar;
    fn alloc(title: &str) -> Touchbar {
        let objc = ObjcAppDelegate::new().share();
        let rust = Box::new(RustTouchbarDelegateWrapper {
            objc: objc.clone(),
            next_item_id: Cell::new(0),
            item_map: BTreeMap::<ItemId, InternalItem>::new(),
            bar_map: BTreeMap::<ItemId, InternalBar>::new(),
        });
        unsafe {
            let ptr: u64 = &*rust as *const RustTouchbarDelegateWrapper as u64;
            let _ = msg_send![rust.objc, setRustWrapper: ptr];
            let objc_title = NSString::alloc(nil).init_str(title);
            let _ = msg_send![rust.objc, setTitle: objc_title];
            let _ = msg_send![objc_title, release];
        }
        return rust
    }
    fn set_icon(&self, image: &str) {
        unsafe {
            let filename = NSString::alloc(nil).init_str(image);
            let objc_image = NSImage::alloc(nil).initWithContentsOfFile_(filename);
            let _:() = msg_send![self.objc, setIcon: objc_image];
            let _ = msg_send![filename, release];
        }
    }

    fn create_bar(&mut self) -> BarId {
        unsafe {
            let ident = self.generate_ident();
            // Create touchbar
            let cls = RRTouchBar::class();
            let bar: *mut Object = msg_send![cls, alloc];
            let bar: *mut objc::runtime::Object = msg_send![bar, init];
            let _ : () = msg_send![bar, setDelegate: self.objc.clone()];
            let internal = InternalBar {
                view: bar,
                ident: ident,
                items: Vec::<ItemId>::new(),
            };
            self.bar_map.insert(bar as u64, internal);
            bar as u64
        }
    }
    fn create_popover_item(&mut self, image: Option<&TouchbarImage>,
                           text: Option<&str>, bar_id: &BarId) -> ItemId {
        unsafe {
            let bar = *bar_id as *mut Object;
            let ident = self.generate_ident();
            let cls = RRPopoverTouchBarItem::class();
            let item: *mut Object = msg_send![cls, alloc];
            let item: *mut Object = msg_send![item, initWithIdentifier: ident];

            let target = (&*self.objc.clone()) as *const ObjcAppDelegate as *mut Object;
            let btn = self.alloc_button(image, text,
                                        target,
                                        sel!(popbar:));
            let _:() = msg_send![item, setShowsCloseButton: YES];
            let gesture: *mut Object = msg_send![item, makeStandardActivatePopoverGestureRecognizer];
            let _:() = msg_send![btn, addGestureRecognizer: gesture];
            let _:() = msg_send![item, setCollapsedRepresentation: btn];
            let _:() = msg_send![item, setPopoverTouchBar: bar];
            let _:() = msg_send![item, setPressAndHoldTouchBar: bar];

            let internal = InternalItem {
                _type: ItemType::Popover,
                view: item,
                ident: ident,
                control: Some(btn),
                scrubber: None,
                button_cb: None,
                slider_cb: None,
                swipe_cb: None,
                tap_cb: None,
                child_bar: Some(bar as ItemId),
            };
            self.item_map.insert(item as u64, internal);
            item as u64
        }
    }
    fn add_items_to_bar(&mut self, bar_id: &BarId, items: Vec<ItemId>) {
        unsafe {
            let cls = Class::get("NSMutableArray").unwrap();
            let idents: *mut Object = msg_send![cls, alloc];
            let idents: *mut Object = msg_send![idents, initWithCapacity: items.len()];
            for item in items {
                if let Some(ident) = self.find_ident(&item) {
                    let ident = ident as *mut Object;
                    let _ : () = msg_send![idents, addObject: ident];
                    self.bar_map.get_mut(&bar_id).unwrap().items.push(item);
                }
            }
            let bar = *bar_id as *mut Object;
            let _ : () = msg_send![bar, setDefaultItemIdentifiers: idents];
            let _ = msg_send![idents, release];
        }
    }
    fn set_bar_as_root(&mut self, bar_id: BarId) {
        unsafe {
            let old_bar: *mut Object = msg_send![self.objc, groupTouchBar];
            if old_bar != nil {
                let cls = Class::get("NSTouchBar").unwrap();
                msg_send![cls, minimizeSystemModalFunctionBar: old_bar];
                msg_send![cls, dismissSystemModalFunctionBar: old_bar];
                self.free_bar_allocations(old_bar);
                msg_send![old_bar, release];
            }
            let _ : () = msg_send![self.objc, setGroupTouchBar: bar_id];
            let ident = self.find_bar_ident(&bar_id).unwrap();
            let _ : () = msg_send![self.objc, setGroupIdent: ident];
            let _: () = msg_send![self.objc, applicationDidFinishLaunching: 0];
        }
    }
    fn create_label(&mut self, text: &str) -> ItemId {
        unsafe {
            let frame = NSRect::new(NSPoint::new(0., 0.), NSSize::new(0., 40.));
            let cls = Class::get("NSTextField").unwrap();
            let label: *mut Object = msg_send![cls, alloc];
            let label: *mut Object = msg_send![label, initWithFrame: frame];
            RustTouchbarDelegateWrapper::set_label_font_for_text(label, text);
            let _:() = msg_send![label, setEditable: NO];
            let cell: *mut Object = msg_send![label, cell];
            let _:() = msg_send![cell, setWraps: NO];
            let text = NSString::alloc(nil).init_str(text);
            let _:() = msg_send![label, setStringValue: text];
            let _ = msg_send![text, release];

            let ident = self.generate_ident();
            let cls = RRCustomTouchBarItem::class();
            let item: *mut Object = msg_send![cls, alloc];
            let item: *mut Object = msg_send![item, initWithIdentifier: ident];
            msg_send![item, setView: label];

            let internal = InternalItem {
                _type: ItemType::Label,
                view: item,
                ident: ident,
                control: Some(label),
                scrubber: None,
                button_cb: None,
                slider_cb: None,
                swipe_cb: None,
                tap_cb: None,
                child_bar: None
            };
            self.item_map.insert(item as u64, internal);
            item as u64
        }
    }
    fn update_label(&mut self, label_id: &ItemId, text: &str) {
        unsafe {
            let item: *mut Object = *label_id as *mut Object;
            let label: *mut Object = msg_send![item, view];
            RustTouchbarDelegateWrapper::set_label_font_for_text(label, text);
            let text = NSString::alloc(nil).init_str(text);
            let _:() = msg_send![label, setStringValue: text];
            let _ = msg_send![text, release];
        }
    }
    fn update_label_width(&mut self, label_id: &ItemId, width: u32) {
        unsafe {
            //let _ = msg_send![label, setAutoresizingMask: 0];
            //let _ = msg_send![label, setFrameSize: NSSize::new(600., 10.)];
            //let _ = msg_send![label, setBoundsSize: NSSize::new(600., 10.)];
            //let _ = msg_send![item, setFrameSize: NSSize::new(600., 30.)];
            //let _ = msg_send![label, setPreferredMaxLayoutWidth: 500.];
            //let constraints: *mut Object = msg_send![label, constraints];
            //let count: u32 = msg_send![constraints, count];
            //info!("CONSTRAINTS: {}", count);
            let item: *mut Object = *label_id as *mut Object;
            let label: *mut Object = msg_send![item, view];
            let anchor: *mut Object = msg_send![label, widthAnchor];
            let constraint: *mut Object = msg_send![anchor, constraintEqualToConstant: width as f64];
            let _ = msg_send![constraint, setActive: YES];
        }
    }

    fn create_text_scrubber(&mut self, data: Rc<TScrubberData>) -> ItemId {
        unsafe {
            let ident = self.generate_ident();
            let cls = RRCustomTouchBarItem::class();
            let item: *mut Object = msg_send![cls, alloc];
            let item: *mut Object = msg_send![item, initWithIdentifier: ident];

            // note: frame is ignored, but must be provided.
            let frame = NSRect::new(NSPoint::new(0., 0.), NSSize::new(0., 30.));
            let cls = RRScrubber::class();
            let scrubber: *mut Object = msg_send![cls, alloc];
            let scrubber: *mut Object = msg_send![scrubber, initWithFrame: frame];

            let cls = Class::get("NSScrubberSelectionStyle").unwrap();
            let style: *mut Object = msg_send![cls, outlineOverlayStyle];

            let cls = Class::get("NSScrubberTextItemView").unwrap();
            let _:() = msg_send![scrubber, registerClass: cls forItemIdentifier: ident];
            let _:() = msg_send![scrubber, setDelegate: self.objc.clone()];
            let _:() = msg_send![scrubber, setDataSource: self.objc.clone()];
            let _:() = msg_send![scrubber, setSelectionOverlayStyle: style];
            let _:() = msg_send![scrubber, setMode: 1]; // NSScrubberModeFree
            let _:() = msg_send![item, setView: scrubber];

            let internal = InternalItem {
                _type: ItemType::Scrubber,
                view: item,
                ident: ident,
                control: Some(scrubber),
                scrubber: Some(data),
                button_cb: None,
                slider_cb: None,
                swipe_cb: None,
                tap_cb: None,
                child_bar: None,
            };
            self.item_map.insert(item as u64, internal);
            item as u64
        }
    }
    fn select_scrubber_item(&mut self, scrub_id: &ItemId, index: u32) {
        unsafe {
            let item = *scrub_id as *mut Object;
            let scrubber: *mut Object = msg_send![item, view];
            let _:() = msg_send![scrubber, setSelectedIndex: index];
        }
    }
    fn refresh_scrubber(&mut self, scrub_id: &ItemId) {
        unsafe {
            let item = *scrub_id as *mut Object;
            let scrubber: *mut Object = msg_send![item, view];
            let sel_idx: i32 = msg_send![scrubber, selectedIndex];
            let pre_count: i32 = msg_send![scrubber, numberOfItems];
            let _:() = msg_send![scrubber, reloadData];
            let post_count: i32 = msg_send![scrubber, numberOfItems];
            // reload clears the selected item.  re-select it.
            if sel_idx >= 0 && pre_count == post_count {
                let _:() = msg_send![scrubber, setSelectedIndex: sel_idx];
            }
        }
    }

    fn add_item_swipe_gesture(&mut self, item_id: &ItemId, cb: SwipeCb) {
        unsafe {
            let item = *item_id as *mut Object;
            let view: *mut Object = msg_send![item, view];
            if view == nil {
                return;
            }
            msg_send![view, setAllowedTouchTypes: 1]; // NSTouchTypeMaskDirect
            let cls = Class::get("NSPanGestureRecognizer").unwrap();
            let gesture: *mut Object = msg_send![cls, alloc];
            let gesture: *mut Object = msg_send![gesture,
                                                 initWithTarget: self.objc.clone()
                                                 action: sel!(swipeGesture:)];
            msg_send![gesture, setAllowedTouchTypes: 1]; // NSTouchTypeMaskDirect
            msg_send![view, addGestureRecognizer: gesture];
            msg_send![gesture, release];
            let mut internal_item = self.item_map.remove(item_id).unwrap();
            internal_item.swipe_cb = Some(cb);
            self.item_map.insert(*item_id, internal_item);
        }
    }

    fn add_item_tap_gesture(&mut self, item_id: &ItemId, taps: u32,
                            fingers: u32, cb: ButtonCb) {
        unsafe {
            let item = *item_id as *mut Object;
            let view: *mut Object = msg_send![item, view];
            if view == nil {
                return;
            }
            msg_send![view, setAllowedTouchTypes: 1]; // NSTouchTypeMaskDirect
            let cls = Class::get("NSClickGestureRecognizer").unwrap();
            let gesture: *mut Object = msg_send![cls, alloc];
            let gesture: *mut Object = msg_send![gesture,
                                                 initWithTarget: self.objc.clone()
                                                 action: sel!(tapGesture:)];
            msg_send![gesture, setAllowedTouchTypes: 1]; // NSTouchTypeMaskDirect
            msg_send![gesture, setNumberOfTouchesRequired: fingers];
            msg_send![gesture, setNumberOfClicksRequired: taps];
            msg_send![view, addGestureRecognizer: gesture];
            msg_send![gesture, release];
            let mut internal_item = self.item_map.remove(item_id).unwrap();
            internal_item.tap_cb = Some(cb);
            self.item_map.insert(*item_id, internal_item);
        }
    }

    fn create_spacer(&mut self, space: SpacerType) -> ItemId {
        unsafe {
            let s = match space {
                SpacerType::Small =>
                    NSString::alloc(nil).init_str("NSTouchBarItemIdentifierFixedSpaceSmall"),
                SpacerType::Large =>
                    NSString::alloc(nil).init_str("NSTouchBarItemIdentifierFixedSpaceLarge"),
                SpacerType::Flexible =>
                    NSString::alloc(nil).init_str("NSTouchBarItemIdentifierFlexibleSpace"),
            };
            // This is really stupid, since it's just a string, but to fit into the
            // rest of the system we go ahead and allocate a whole internal item for it.

            // And since it doesn't have an ident, just use it as its own ident.  Both
            // the view and ident will be sent a release at shutdown, so retain it an
            // extra time here to keep the references balanced.
            let _ = msg_send![s, retain];

            let internal = InternalItem {
                _type: ItemType::Spacer,
                view: s,
                ident: s as u64,
                control: None,
                scrubber: None,
                button_cb: None,
                slider_cb: None,
                swipe_cb: None,
                tap_cb: None,
                child_bar: None,
            };
            self.item_map.insert(s as u64, internal);
            s as ItemId
        }
    }

    fn create_image_from_path(&mut self, path: &str) -> TouchbarImage {
        unsafe {
            let filename = NSString::alloc(nil).init_str(path);
            let objc_image = NSImage::alloc(nil).initWithContentsOfFile_(filename);
            let _ = msg_send![filename, release];
            objc_image as TouchbarImage
        }
    }

    fn create_image_from_template(&mut self, template: ImageTemplate) -> TouchbarImage {
        unsafe {
            let cls = Class::get("NSImage").unwrap();
            let image: *mut Object = msg_send![cls, imageNamed: ImageTemplate::objc(template)];
            let _ = msg_send![image, retain];
            image as TouchbarImage
        }
    }

    fn create_button(&mut self, image: Option<&TouchbarImage>, text: Option<&str>, cb: ButtonCb) -> ItemId {
        unsafe {
            let ident = self.generate_ident();
            let target = (&*self.objc.clone()) as *const ObjcAppDelegate as *mut Object;
            let btn = self.alloc_button(image, text,
                                        target,
                                        sel!(button:));
            let cls = RRCustomTouchBarItem::class();
            let item: *mut Object = msg_send![cls, alloc];
            let item: *mut Object = msg_send![item, initWithIdentifier: ident];
            msg_send![item, setView: btn];

            let internal = InternalItem {
                _type: ItemType::Button,
                view: item,
                ident: ident,
                control: Some(btn),
                scrubber: None,
                button_cb: Some(cb),
                slider_cb: None,
                swipe_cb: None,
                tap_cb: None,
                child_bar: None,
            };
            self.item_map.insert(item as u64, internal);
            item as u64
        }
    }

    fn update_button(&mut self, item: &ItemId, image: Option<&TouchbarImage>, text: Option<&str>) {
        unsafe {
            let item = *item as *mut Object;
            let btn: *mut Object = msg_send![item, view];
            if let Some(image) = image {
                let image = *image as *mut Object;
                let _ = msg_send![btn, setImage: image];
                let _ = msg_send![image, release];
            }
            if let Some(text) = text {
                let objc_text = NSString::alloc(nil).init_str(text);
                let _ = msg_send![btn, setTitle: objc_text];
                let _ = msg_send![objc_text, release];
            }
        }
    }

    fn update_button_width(&mut self, button_id: &ItemId, width: u32) {
        unsafe {
            let item: *mut Object = *button_id as *mut Object;
            let control: *mut Object = msg_send![item, view];
            let anchor: *mut Object = msg_send![control, widthAnchor];
            let constraint: *mut Object = msg_send![anchor, constraintEqualToConstant: width as f64];
            let _ = msg_send![constraint, setActive: YES];
        }
    }

    fn create_slider(&mut self, min: f64, max: f64,
                     label: Option<&str>,
                     continuous: bool, cb: SliderCb) -> ItemId {
        unsafe {
            let ident = self.generate_ident();
            let cls = RRSliderTouchBarItem::class();
            let item: *mut Object = msg_send![cls, alloc];
            let item: *mut Object = msg_send![item, initWithIdentifier: ident];
            let slider: *mut Object = msg_send![item, slider];
            if let Some(label) = label {
                let objc_text: *mut Object = NSString::alloc(nil).init_str(label);
                msg_send![item, setLabel: objc_text];
                msg_send![objc_text, release];
            }
            msg_send![slider, setMinValue: min];
            msg_send![slider, setMaxValue: max];
            msg_send![slider, setContinuous: continuous];
            msg_send![item, setTarget: self.objc.clone()];
            msg_send![item, setAction: sel!(slider:)];

            let internal = InternalItem {
                _type: ItemType::Slider,
                view: item,
                ident: ident,
                control: Some(slider),
                scrubber: None,
                button_cb: None,
                slider_cb: Some(cb),
                swipe_cb: None,
                tap_cb: None,
                child_bar: None,
            };
            self.item_map.insert(item as u64, internal);
            item as u64
        }
    }
    fn update_slider(&mut self, id: &ItemId, value: f64) {
        unsafe {
            let item = *id as *mut Object;
            let slider: *mut Object = msg_send![item, slider];
            let _:() = msg_send![slider, setDoubleValue: value];
        }
    }
}

// Below here defines a new native Obj-C class.
//
// See rustc-objc-foundation project by SSheldon, examples/custom_class.rs
// https://github.com/SSheldon/rust-objc-foundation/blob/master/examples/custom_class.rs
pub enum ObjcAppDelegate {}
impl ObjcAppDelegate {}

unsafe impl Message for ObjcAppDelegate { }

static OBJC_SUBCLASS_REGISTER_CLASS: Once = ONCE_INIT;

impl INSObject for ObjcAppDelegate {
    fn class() -> &'static Class {
        OBJC_SUBCLASS_REGISTER_CLASS.call_once(|| {
            let superclass = NSObject::class();
            let mut decl = ClassDecl::new("ObjcAppDelegate", superclass).unwrap();
            decl.add_ivar::<u64>("_rust_wrapper");
            decl.add_ivar::<u64>("_group_bar");
            decl.add_ivar::<u64>("_group_id");
            decl.add_ivar::<u64>("_tray_item");
            decl.add_ivar::<u64>("_title");
            decl.add_ivar::<u64>("_icon");

            extern fn objc_set_title(this: &mut Object, _cmd: Sel, ptr: u64) {
                unsafe {this.set_ivar("_title", ptr);}
            }
            extern fn objc_set_rust_wrapper(this: &mut Object, _cmd: Sel, ptr: u64) {
                unsafe {this.set_ivar("_rust_wrapper", ptr);}
            }
            extern fn objc_group_touch_bar(this: &mut Object, _cmd: Sel) -> u64 {
                unsafe {*this.get_ivar("_group_bar")}
            }
            extern fn objc_set_group_touch_bar(this: &mut Object, _cmd: Sel, bar: u64) {
                unsafe {
                    this.set_ivar("_group_bar", bar);
                }
            }
            extern fn objc_set_group_ident(this: &mut Object, _cmd: Sel, bar: u64) {
                unsafe {this.set_ivar("_group_id", bar);}
            }
            extern fn objc_set_icon(this: &mut Object, _cmd: Sel, icon: u64) {
                unsafe {this.set_ivar("_icon", icon);}
            }
            extern fn objc_number_of_items_for_scrubber(this: &mut Object, _cmd: Sel,
                                                        scrub: u64) -> u32 {
                unsafe {
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);
                    if let Some(ref scrubber) = wrapper.find_scrubber(scrub) {
                        if let Some(ref cbs) = wrapper.find_scrubber_callbacks(scrub) {
                            let count = cbs.count(*scrubber);
                            return count;
                        }
                    }
                    0
                }
            }
            extern fn objc_scrubber_view_for_item_at_index(this: &mut Object, _cmd: Sel,
                                                           scrub: u64, idx: u32) -> u64 {
                unsafe {
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);
                    let scrubber = scrub as *mut Object;
                    if let Some(ref item) = wrapper.find_scrubber(scrub) {
                        if let Some(ref cbs) = wrapper.find_scrubber_callbacks(scrub) {
                            let ident = wrapper.find_ident_from_control(&scrub).unwrap() as
                                *mut Object;
                            let view: *mut Object = msg_send![scrubber,
                                                              makeItemWithIdentifier:ident
                                                              owner:nil];
                            let text = cbs.text(*item, idx);
                            let text_field: *mut Object = msg_send![view, textField];
                            let objc_text: *mut Object = NSString::alloc(nil).init_str(&text);
                            let _:() = msg_send![text_field, setStringValue: objc_text];
                            let _ = msg_send![objc_text, release];
                            return view as u64;
                        }
                    }
                    0
                }
            }
            extern fn objc_scrubber_layout_size_for_item_at_index(this: &mut Object, _cmd: Sel,
                                                                  scrub: u64,
                                                                  _layout: u64, idx: u32) -> NSSize {
                unsafe {
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);
                    if let Some(ref item) = wrapper.find_scrubber(scrub) {
                        if let Some(ref cbs) = wrapper.find_scrubber_callbacks(scrub) {
                            let width = cbs.width(*item, idx);
                            return NSSize::new(width as f64, 30.);
                        }
                    }
                    NSSize::new(0., 30.)
                }
            }
            extern fn objc_scrubber_did_select_item_at_index(this: &mut Object, _cmd: Sel,
                                                             scrub: u64, idx: u32) {
                unsafe {
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);
                    if let Some(ref item) = wrapper.find_scrubber(scrub) {
                        if let Some(ref cbs) = wrapper.find_scrubber_callbacks(scrub) {
                            cbs.touch(*item, idx);
                        }
                    }
                }
            }
            extern fn objc_popbar(this: &mut Object, _cmd: Sel, sender: u64) {
                unsafe {
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);

                    let item = wrapper.find_popover(sender) .unwrap_or(0) as *mut Object;
                    let ident = wrapper.find_ident_from_control(&sender) .unwrap_or(0) as *mut Object;
                    if item == nil || ident == nil {
                        return;
                    }
                    let bar: *mut Object = msg_send![item, popoverTouchBar];

                    // Present the request popover.  This must be done instead of
                    // using the popover's built-in showPopover because that pops
                    // _under_ a system function bar.
                    let cls = Class::get("NSTouchBar").unwrap();
                    msg_send![cls,
                              presentSystemModalFunctionBar: bar
                              systemTrayItemIdentifier: ident];
                    let app = NSApp();
                    let _:() = msg_send![app, setTouchBar: nil];
                }
            }
            extern fn objc_button(this: &mut Object, _cmd: Sel, sender: u64) {
                unsafe {
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);
                    if let Some(ref cb) = wrapper.find_button_cb(sender) {
                        // Sender is the button.  Find the owning touchbar item:
                        let item = wrapper.find_view_from_control(&sender).unwrap();
                        cb(&(item as u64));
                    }
                }
            }
            extern fn objc_tap_gesture(this: &mut Object, _cmd: Sel, sender: u64) {
                unsafe {
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);
                    let gesture = sender as *mut Object;
                    let view: *mut Object = msg_send![gesture, view];
                    if let Some(ref cb) = wrapper.find_tap_cb(view as u64) {
                        // Sender is the view.  Find the owning touchbar item:
                        let item = wrapper.find_view_from_control(&(view as u64)).unwrap();
                        cb(&(item as u64));
                    }
                }
            }
            extern fn objc_swipe_gesture(this: &mut Object, _cmd: Sel, sender: u64) {
                unsafe {
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);
                    let gesture = sender as *mut Object;
                    let view: *mut Object = msg_send![gesture, view];
                    let translation: NSPoint = msg_send![gesture,
                                                         translationInView: view];
                    let gesture_state: u32 = msg_send![gesture, state];
                    let state = match gesture_state {
                        // NSGestureRecognizerStateBegan
                        1 => SwipeState::Began,
                        // NSGestureRecognizerStateChanged
                        2 => SwipeState::Changed,
                        // NSGestureRecognizerStateEnded
                        3 => SwipeState::Ended,
                        // NSGestureRecognizerStateCancelled,
                        4 => SwipeState::Cancelled,
                        // NSGestureRecognizerStateFailed
                        5 => SwipeState::Failed,
                        // NSGestureRecognizerStatePossible,
                        _ => SwipeState::Unknown,
                    };
                    if state != SwipeState::Unknown {
                        if let Some(ref cb) = wrapper.find_swipe_cb(view as u64) {
                            // Sender is the view.  Find the owning touchbar item:
                            let item = wrapper.find_view_from_control(&(view as u64)).unwrap();
                            cb(&(item as u64), state, translation.x);
                        }
                    }
                }
            }
            extern fn objc_slider(this: &mut Object, _cmd: Sel, sender: u64) {
                unsafe {
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);
                    if let Some(ref cb) = wrapper.find_slider_cb(sender) {
                        let item = sender as *mut Object;
                        let slider: *mut Object = msg_send![item, slider];
                        let value: f64 = msg_send![slider, doubleValue];
                        cb(&sender, value);
                    }
                }
            }
            extern fn objc_present(this: &mut Object, _cmd: Sel, _sender: u64) {
                unsafe {
                    let ident_int: u64 = *this.get_ivar("_group_id");
                    let bar_int: u64 = *this.get_ivar("_group_bar");
                    let ident = ident_int as *mut Object;
                    let bar = bar_int as *mut Object;
                    let cls = Class::get("NSTouchBar").unwrap();
                    msg_send![cls,
                              presentSystemModalFunctionBar: bar
                              systemTrayItemIdentifier: ident];
                }
            }
            extern fn objc_touch_bar_make_item_for_identifier(this: &mut Object, _cmd: Sel,
                                                              _bar: u64, id_ptr: u64) -> u64 {
                unsafe {
                    // Find the touchbar item matching this identifier in the
                    // Objective-C object map of the Rust wrapper class, and
                    // return it if found.
                    let ptr: u64 = *this.get_ivar("_rust_wrapper");
                    let wrapper = &mut *(ptr as *mut RustTouchbarDelegateWrapper);
                    if let Some(obj) = wrapper.find_view(id_ptr) {
                        return obj as u64;
                    }
                }
                0
            }
            extern fn objc_application_did_finish_launching(this: &mut Object, _cmd: Sel, _notification: u64) {
                unsafe {
                    DFRSystemModalShowsCloseBoxWhenFrontMost(YES);

                    let old_item_ptr: u64 = *this.get_ivar("_tray_item");
                    let old_item = old_item_ptr as *mut Object;
                    if old_item != nil {
                        let cls = Class::get("NSTouchBarItem").unwrap();
                        let _ = msg_send![cls, removeSystemTrayItem: old_item];
                        let _ = msg_send![old_item, release];
                    }

                    let app = NSApp();
                    let _:() = msg_send![app, setTouchBar: nil];

                    let ident_int: u64 = *this.get_ivar("_group_id");
                    let ident = ident_int as *mut Object;
                    let cls = Class::get("NSCustomTouchBarItem").unwrap();
                    let item: *mut Object = msg_send![cls, alloc];
                    msg_send![item, initWithIdentifier:ident];
                    this.set_ivar("_tray_item", item as u64);

                    let cls = Class::get("NSButton").unwrap();
                    let icon_ptr: u64 = *this.get_ivar("_icon");
                    let title_ptr: u64 = *this.get_ivar("_title");
                    let btn: *mut Object;
                    if icon_ptr != (nil as u64) {
                        btn = msg_send![cls,
                                        buttonWithImage:icon_ptr
                                        target:this
                                        action:sel!(present:)];
                    }
                    else {
                        btn = msg_send![cls,
                                        buttonWithTitle:title_ptr
                                        target:this
                                        action:sel!(present:)];
                    }
                    msg_send![item, setView:btn];

                    let cls = Class::get("NSTouchBarItem").unwrap();
                    msg_send![cls, addSystemTrayItem: item];
                    DFRElementSetControlStripPresenceForIdentifier(ident, YES);
                }
            }

            unsafe {
                let f: extern fn(&mut Object, Sel, u64, u64) -> u64 = objc_touch_bar_make_item_for_identifier;
                decl.add_method(sel!(touchBar:makeItemForIdentifier:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_application_did_finish_launching;
                decl.add_method(sel!(applicationDidFinishLaunching:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_present;
                decl.add_method(sel!(present:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_button;
                decl.add_method(sel!(button:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_tap_gesture;
                decl.add_method(sel!(tapGesture:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_swipe_gesture;
                decl.add_method(sel!(swipeGesture:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_slider;
                decl.add_method(sel!(slider:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_popbar;
                decl.add_method(sel!(popbar:), f);

                let f: extern fn(&mut Object, Sel) -> u64 = objc_group_touch_bar;
                decl.add_method(sel!(groupTouchBar), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_set_group_touch_bar;
                decl.add_method(sel!(setGroupTouchBar:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_set_group_ident;
                decl.add_method(sel!(setGroupIdent:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_set_icon;
                decl.add_method(sel!(setIcon:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_set_rust_wrapper;
                decl.add_method(sel!(setRustWrapper:), f);

                let f: extern fn(&mut Object, Sel, u64) = objc_set_title;
                decl.add_method(sel!(setTitle:), f);

                // Scrubber delegates
                let f: extern fn(&mut Object, Sel, u64) -> u32 = objc_number_of_items_for_scrubber;
                decl.add_method(sel!(numberOfItemsForScrubber:), f);
                let f: extern fn(&mut Object, Sel, u64, u32) -> u64 = objc_scrubber_view_for_item_at_index;
                decl.add_method(sel!(scrubber:viewForItemAtIndex:), f);
                let f: extern fn(&mut Object, Sel, u64, u64, u32) -> NSSize = objc_scrubber_layout_size_for_item_at_index;
                decl.add_method(sel!(scrubber:layout:sizeForItemAtIndex:), f);
                let f: extern fn(&mut Object, Sel, u64, u32) = objc_scrubber_did_select_item_at_index;
                decl.add_method(sel!(scrubber:didSelectItemAtIndex:), f);
            }

            decl.register();
        });

        Class::get("ObjcAppDelegate").unwrap()
    }
}
