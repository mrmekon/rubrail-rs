use std::rc::Rc;

/// Reference to a horizontal bar created by a `TTouchbar`
///
/// A `BarId` is returned any time a bar is created, where a bar is a horizontal
/// view that can have smaller UI elements added to it, and can eventually be
/// displayed on the Touch Bar hardware.
///
/// The two types of bars are the _root bar_, which appears when you first open
/// the Touch Bar by clicking on its icon in the Control Strip, and _popover
/// bars_, which behave identically but are presented by clicking on buttons
/// on a bar or popover.  Popovers provide the mechanism for recursive menus.
///
/// # Memory Allocation
///
/// Memory is allocated for a bar when it is created, and is not released until
/// the bar is made the root bar, or a popover of the root bar, _enabled_, _and
/// subsequently replaced by another bar_.  That is, memory is deallocated
/// recursively when an available Touch Bar menu is replaced.  If a menu is
/// never registered as the active menu, then it will _never be deallocated_.
/// `BarId` does _not_ implement the Drop trait, and does _not_ deallocate any
/// memory when it falls out of scope.
pub type BarId = u64;

/// Reference to an item that can be added to a bar created by a `TTouchbar`
///
/// An `ItemId` is returned when UI elements are created, and can then be
/// assigned to bars by associating a list of `ItemId`s with a `BarId`.
///
/// # Memory Allocation
///
/// Memory is allocated when an item is created, and is not released until the
/// parent bar that owns it is released.  This means it follows the same memory
/// management cycle as `BarId` -- items are not released unless they are
/// assigned to a bar, that bar is registered as the root bar, and then that bar
/// is replaced.  `ItemId` does not implement the Drop trait, and does _not_
/// deallocate memory when it falls out of scope.
pub type ItemId = u64;

/// A callback that is called when a button on a Touch Bar is pressed
///
/// `ButtonCb` is expected to be a Boxed closure, and it receives the
/// `ItemId` of the button that is pressed.
///
/// # Arguments
///
/// * first - `ItemId` of the button that was pressed
pub type ButtonCb = Box<Fn(ItemId)>;

/// A callback that is called when the value of a slide on a Touch Bar changes
///
/// 'SliderCb' is expected to be a Boxed closure, and it receives the `ItemId`
/// of the slider that changed, and the new value of the slider as a float.
///
/// # Arguments
///
/// * first - `ItemId` of the slider that was changed
/// * second - Current value of the slider
pub type SliderCb = Box<Fn(ItemId, f64)>;

/// An allocated image that can be added to items
///
/// A `TouchbarImage` can be created from a path to a file or from a standard
/// Apple template image, and then registered with Touch Bar items that support
/// images, such as buttons and popovers.
pub type TouchbarImage = u64;

/// Identifiers for Apple's standard button image templates
#[allow(missing_docs)]
pub enum ImageTemplate {
    AlarmTemplate,
    RewindTemplate,
    FastForwardTemplate,
    PlayTemplate,
    PauseTemplate,
    PlayPauseTemplate,
    ListViewTemplate,
    AudioOutputVolumeMediumTemplate,
    GoUpTemplate,
}

/// Identifiers for the type of spacing available between items
pub enum SpacerType {
    /// "Small" space, defined by Apple
    Small,
    /// "Large" space, defined by Apple
    Large,
    /// Flexible space, grows and shrinks as it can/needs
    Flexible
}

/// The callback API for managing data in a Scrubber
///
/// The Touch Bar supports a UI element called a 'scrubber', which is a
/// horizontally scrolling widget filled with items which can be dynamically
/// changed, and selected.  This is the primary interface for choosing from
/// a list of (possibly dynamic) options.
///
/// Since the contents of a scrubber are dynamic, the scrubber fills in its
/// data on request through a series of callbacks.  A type that implements
/// `TScrubberData` provides all of the callbacks that a scrubber needs to
/// present its options.
///
/// See the examples for usage.
pub trait TScrubberData {
    /// Returns the number of items in the scrubber
    ///
    /// # Arguments
    ///
    /// * `item` - The `ItemId` of the interacting scrubber
    fn count(&self, item: ItemId) -> u32;

    /// Returns the text of the given index in the scrubber
    ///
    /// # Arguments
    ///
    /// * `item` - The `ItemId` of the interacting scrubber
    /// * `idx` - The index of the relevant item in the scrubber
    fn text(&self, item: ItemId, idx: u32) -> String;

    /// Returns the width (in pixels) of the given index in the scrubber
    ///
    /// # Arguments
    ///
    /// * `item` - The `ItemId` of the interacting scrubber
    /// * `idx` - The index of the relevant item in the scrubber
    fn width(&self, item: ItemId, idx: u32) -> u32;

    /// Called when the given index in the scrubber is selected
    ///
    /// # Arguments
    ///
    /// * `item` - The `ItemId` of the interacting scrubber
    /// * `idx` - The index of the relevant item in the scrubber
    fn touch(&self, item: ItemId, idx: u32);
}

/// API for creating, managing, and getting feedback from Touch Bar UIs
///
/// `TTouchbar` is the trait that defines the API for all interactions with the
/// Touch Bar.
///
/// See the documentation of the [`Touchbar`](type.Touchbar.html) type for
/// information on the default implementation.
///
/// An additional 'dummy' implementation is available by building Rubrail with
/// the `--no-default-features` flags.  This allows you to build an application
/// that assumes a Touch Bar exists, but to remove it without code changes on
/// platforms that don't have a Touch Bar, or for distributing through official
/// Apple channels which don't permit private API usage.
///
pub trait TTouchbar {
    /// A concrete implementation of TTouchbar
    type T: TTouchbar;

    /// Allocate a new Touch Bar interface
    ///
    /// This allocates a new Touch Bar interface, but does not cause anything
    /// to be displayed yet.
    ///
    /// # Arguments
    ///
    /// * `title` - The text to display on the icon in the Control Strip area
    ///   of the Touch Bar.  The title is only displayed if no icon is set.
    ///   Note that any length is permitted, but the width of the icon is
    ///   controlled by the OS, and typically truncates anything over 5 chars.
    ///
    /// # Returns
    ///
    /// A newly allocated instance of the type.
    ///
    fn alloc(title: &str) -> Self::T;

    /// Set an icon to display in the Control Strip
    ///
    /// It is preferrable to associate your Touch Bar menus with an icon instead
    /// of a text string.  This registers an icon, which will be displayed
    /// in the Control Strip region of the Touch Bar when you register a bar.
    ///
    /// This function takes a path to an image, which should follow all of the
    /// standard Apple guidelines for icons on Retina displays.  This typically
    /// means a PNG that is 40px x 40px @ 150dpi, or a multiple of this (80px x
    /// 80px @ 300dpi).
    ///
    /// Note that Rubrail must run from a Mac app bundle, which means the icon
    /// will typically be in the bundle's Resources directory, and you must
    /// take care to provide the path to it correctly.
    ///
    /// # Arguments
    ///
    /// * `image` - Full path to an image following the Apple icon guidelines
    ///
    fn set_icon(&self, image: &str) {}

    /// Create a new horizontal bar UI
    ///
    /// This allocates a bar container, which will be either the root bar or
    /// a 'popover' for recursive menus.  It contains no items and is not
    /// displayed or registered when it is created.
    ///
    /// # Returns
    ///
    /// A newly allocated, empty bar.
    fn create_bar(&mut self) -> BarId { 0 }

    /// Adds a group of ordered items to a bar
    ///
    /// This adds an array of allocated items, in the order provided, to an
    /// allocated bar.  This does not cause the bar to be displayed or
    /// registered.
    ///
    /// # Arguments
    ///
    /// * `bar_id` - Bar to add the items to
    /// * `items` - Vector of items to add to the bar
    ///
    fn add_items_to_bar(&mut self, bar_id: &BarId, items: Vec<ItemId>) {}

    /// Sets the given bar as the 'root' bar in the Control Strip
    ///
    /// Registers the given bar as the 'root' bar.  This creates an icon in the
    /// Control Strip region of the Touch Bar (the section on the right side),
    /// and causes this bar to be displayed when the icon is pressed.
    ///
    /// This function causes the UI to be updated and the bar to become useable
    /// by the user.
    ///
    /// # Arguments
    ///
    /// * `bar_id` - The bar to present when the Control Strip icon is pressed
    ///
    fn set_bar_as_root(&mut self, bar_id: BarId) {}

    /// Create a button to open a 'popover' submenu.
    ///
    /// Creates a button UI element that, when pressed, recursively opens
    /// another bar as a submenu.  Popovers allow you to create infinitely
    /// nested heirarchies of touch bar menus.  Bars registered as popovers
    /// can themselves contain more popover items.
    ///
    /// All buttons accept an image, text, or both.  If both are provided, they
    /// will both be displayed at the same time.
    ///
    /// # Arguments
    ///
    /// * `image` - An image allocated with a `create_image_*` function
    /// * `text` - Text to display on the button
    /// * `bar_id` - Bar to present when this button is pressed
    ///
    /// # Returns
    ///
    /// A newly allocated item which can be added to a bar.
    fn create_popover_item(&mut self, image: Option<&TouchbarImage>,
                           text: Option<&str>, bar_id: &BarId) -> ItemId {0}

    /// Create a new label
    ///
    /// Creates a text label, which simply displays a line of non-interactive
    /// text.  Newlines are permitted, though only two lines will render
    /// properly.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to display in label
    ///
    /// # Returns
    ///
    /// A newly allocated label item
    fn create_label(&mut self, text: &str) -> ItemId {0}

    /// Changes the text in an existing label
    ///
    /// This changes the text of an existing label, allowing updating of text
    /// displays without rebuilding the whole bar.
    ///
    /// # Arguments
    ///
    /// * `label_id` - Label item to change
    /// * `text` - New text to display in the existing label
    ///
    fn update_label(&mut self, label_id: &ItemId, text: &str) {}

    /// Changes the width of an existing label
    ///
    /// Set a fixed width for a label, in pixels.
    ///
    /// # Arguments
    ///
    /// * `label_id` - Label item to change
    /// * `width` - New width of label, in pixels
    ///
    fn update_label_width(&mut self, label_id: &ItemId, width: u32) {}

    /// Create a horizontally scrolling 'scrubber' of text
    ///
    /// Creates a Scrubber, which is a  horizontally scrolling widget filled
    /// with items which can be dynamically changed, and selected.  This is the
    /// primary interface for choosing from a list of (possibly dynamic) options.
    ///
    /// Scrubbers are filled with data using a collection of callbacks.  This
    /// is implemented by the [TScrubberData](trait.TScrubberData.html) trait,
    /// which allows you to build a custom backing data store for scrubbers.
    ///
    /// At creation, no item is selected.  Call `select_scrubber_item()` after
    /// this to render an item as selected.
    ///
    /// # Arguments
    ///
    /// * `data` - An object implementing the `TScrubberData` trait, wrapped
    ///   in a reference counter (Rc).
    ///
    /// # Returns
    ///
    /// A newly allocated scrubber item
    fn create_text_scrubber(&mut self, data: Rc<TScrubberData>) -> ItemId {0}

    /// Selects the given index in a scrubber
    ///
    /// Marks the given index in the given scrubber as selected, so that item
    /// will be displayed with highlighting.
    ///
    /// # Arguments
    ///
    /// * `scrub_id` - Scrubber to select in
    /// * `index` - Index of the item to mark as selected
    ///
    fn select_scrubber_item(&mut self, scrub_id: &ItemId, index: u32) {}

    /// Inform a scrubber to redraw after a change to its backing data
    ///
    /// If the data store backing a scrubber (`TScrubberData`) has its data
    /// change, this function should be called to inform the scrubber to
    /// redraw.
    ///
    /// # Arguments
    ///
    /// * `scrub_id` - Scrubber to refresh
    ///
    fn refresh_scrubber(&mut self, scrub_id: &ItemId) {}

    /// Create space between items in a bar
    ///
    /// # Arguments
    ///
    /// * `space` - The type of spacer to create
    ///
    /// # Returns
    ///
    /// A newly allocated spacer item that can be added to a bar
    ///
    fn create_spacer(&mut self, space: SpacerType) -> ItemId {0}

    /// Create an image from a file path
    ///
    /// Creates an image that can be assigned to UI items that display them,
    /// like buttons and popovers.
    ///
    /// Specify a relative or absolute path to the image.
    ///
    /// When specifying an image, follow Apple's guidelines for icons on Retina
    /// displays (40px x 40px @ 150dpi PNG recommended), and remember to handle
    /// paths into the app bundle correctly.
    ///
    /// **WARNING** image memory is _deallocated_ after it is assigned to an
    /// item.  Do **not** use the same image twice.  If two buttons will have
    /// the same image, you must allocate the image twice.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to an image file
    ///
    /// # Returns
    ///
    /// A newly allocated image that can be added to an item
    ///
    fn create_image_from_path(&mut self, path: &str) -> TouchbarImage {0}

    /// Create an image from a template
    ///
    /// Creates an image that can be assigned to UI items that display them,
    /// like buttons and popovers.
    ///
    /// See `ImageTemplate` for the supported options.  These templates are
    /// provided by Apple.
    ///
    /// **WARNING** image memory is _deallocated_ after it is assigned to an
    /// item.  Do **not** use the same image twice.  If two buttons will have
    /// the same image, you must allocate the image twice.
    ///
    /// # Arguments
    ///
    /// * `template` - Identifier of the image template to use
    ///
    /// # Returns
    ///
    /// A newly allocated image that can be added to an item
    ///
    fn create_image_from_template(&mut self, template: ImageTemplate) -> TouchbarImage {0}

    /// Create a button that triggers a callback when pressed
    ///
    /// All buttons accept an image, text, or both.  If both are provided, they
    /// will both be displayed at the same time.
    ///
    /// # Arguments
    ///
    /// * `image` - An image allocated with a `create_image_*` function
    /// * `text` - Text to display on the button
    /// * `cb` - Callback to call when the button is pressed
    ///
    /// # Returns
    ///
    /// A newly allocated item which can be added to a bar.
    fn create_button(&mut self, image: Option<&TouchbarImage>, text: Option<&str>, cb: ButtonCb) -> ItemId {0}

    /// Changes the image and/or text of a button
    ///
    /// # Arguments
    ///
    /// * `item` - Button item to change
    /// * `image` - New image to draw on button (optional)
    /// * `text` - New text to draw on button (optional)
    ///
    fn update_button(&mut self, item: &ItemId, image: Option<&TouchbarImage>, text: Option<&str>) {}

    /// Changes the width of an existing button
    ///
    /// Set a fixed width for a button, in pixels.
    ///
    /// # Arguments
    ///
    /// * `button_id` - Button item to change
    /// * `width` - New width of button, in pixels
    ///
    fn update_button_width(&mut self, button_id: &ItemId, width: u32) {}

    /// Create a slider item
    ///
    /// Creates an item that displays as a continuously variable horizontal
    /// slider, reporting its value as a floating point between the provided
    /// minimum and maximum values.
    ///
    /// When the slider value changes, the associated callback is called.  Note
    /// that it triggers frequently as the user slides it, so 'debouncing' or
    /// buffering might be required if high-frequency changes are not desired.
    ///
    /// Newly created sliders default to the minimum value.  If you need to
    /// change this, set the current value with `update_slider()`.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum value (slider all the way left)
    /// * `max` - Maximum value (slider all the way right)
    /// * `continuous` - Whether callback is called while sliding, or only
    /// * `cb` - Callback called when the slider value is changed
    ///   after it is released.
    ///
    /// # Returns
    ///
    /// A newly allocated slider item
    fn create_slider(&mut self, min: f64, max: f64,
                     continuous: bool, cb: SliderCb) -> ItemId {0}

    /// Update the current position of a slider
    ///
    /// Sets the current value of an existing slider.
    ///
    /// # Arguments
    ///
    /// * `id` - Slider item to update
    /// * `value` - New value of the slider.  Must be between the min and max
    ///   specified when the slider was created.
    ///
    fn update_slider(&mut self, id: &ItemId, value: f64) {}
}
