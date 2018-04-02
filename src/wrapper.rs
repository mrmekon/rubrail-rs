extern crate objc;
extern crate objc_foundation;

use objc::runtime::Class;

#[cfg(feature = "objc_wrapper")]
use objc::runtime::{Object, Sel};
#[cfg(feature = "objc_wrapper")]
use std::sync::{Once, ONCE_INIT};
#[cfg(feature = "objc_wrapper")]
use objc::Message;
#[cfg(feature = "objc_wrapper")]
use objc::declare::ClassDecl;
#[cfg(feature = "objc_wrapper")]
use self::objc_foundation::INSObject;

/// Wrap an Objective-C class in a subclass that tracks allocations
///
/// This creates an Objective-C wrapper class which can be used during
/// development to print or set breakpoints on object retain, release,
/// or dealloc of certain Obj-C objects.
///
/// Enable by compiling with the `objc_wrapper` feature.  By default
/// it only adds printing when an object deallocation begins, and
/// provides nice places to hook breakpoints with lldb.
///
/// # Arguments
///
/// * `$newclass` - Name of the new Obj-C class to create
///
/// * `$superclass` - Name of the existing Obj-C class to wrap
///
/// * `$unique_newclass` - Any unique variable name that is different from
///   $newclass.  This is just used to manage one-time initialization,
///   and is manual because `concat_idents!()` is unstable.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate rubrail;
/// #[macro_use]
/// extern crate objc;
/// use objc::runtime::Class;
/// use objc::runtime::Object;
///
/// fn main() {
///   objc_subclass!(MyNSObject, NSObject, MYNSOBJECT_CLASS);
///   let cls = MyNSObject::class();
///   unsafe {
///     let obj: *mut Object = msg_send![cls, alloc];
///     let obj: *mut Object = msg_send![obj, init];
///   }
/// }
/// ```
#[cfg(not(feature = "objc_wrapper"))]
#[macro_export]
macro_rules! objc_subclass {
    ( $newclass:ident, $superclass:ident, $unique_newclass:ident ) => {
        pub struct $newclass {}
        impl $newclass {
            pub fn class() -> &'static Class {
                Class::get(stringify!($superclass)).unwrap()
            }
        }
    }
}

#[cfg(feature = "objc_wrapper")]
#[macro_export]
macro_rules! objc_subclass {
    ( $newclass:ident, $superclass:ident, $unique_newclass:ident ) => {
        pub enum $newclass {}
        impl $newclass {}
        unsafe impl Message for $newclass { }
        static $unique_newclass: Once = ONCE_INIT;
        impl $newclass {
            fn str_name(this: &mut Object) -> String {
                let ptr = this as *mut Object;
                format!("{}({:x})", stringify!($newclass), ptr as u64)
            }
        }
        impl INSObject for $newclass {
            fn class() -> &'static Class {
                $unique_newclass.call_once(|| {
                    let superclass = Class::get(stringify!($superclass)).unwrap();
                    let mut decl = ClassDecl::new(stringify!($newclass), superclass).unwrap();
                    decl.add_ivar::<u64>("_retain_count");
                    extern fn objc_retain(this: &mut Object, _cmd: Sel) -> *mut Object {
                        unsafe {
                            //info!("{} retain", $newclass::str_name(this));
                            let superclass = Class::get(stringify!($superclass)).unwrap();
                            let obj: *mut Object = msg_send![super(this, superclass), retain];
                            //let count: u32 = msg_send![super(this, superclass), retainCount];
                            //info!("{} retain done! {}", $newclass::str_name(this), count);
                            obj
                        }
                    }
                    extern fn objc_release(this: &mut Object, _cmd: Sel) {
                        unsafe {
                            //info!("{} release", $newclass::str_name(this));
                            let superclass = Class::get(stringify!($superclass)).unwrap();
                            let _: () = msg_send![super(this, superclass), release];
                            //let count: u32 = msg_send![super(this, superclass), retainCount];
                            //info!("{} release done! {}", $newclass::str_name(this), count);
                        }
                    }
                    extern fn objc_dealloc(this: &mut Object, _cmd: Sel) {
                        unsafe {
                            info!("{} dealloc", $newclass::str_name(this));
                            let superclass = Class::get(stringify!($superclass)).unwrap();
                            let _: () = msg_send![super(this, superclass), dealloc];
                            //info!("{} dealloc done", $newclass::str_name(this));
                        }
                    }
                    unsafe {
                        let f: extern fn(&mut Object, Sel) -> *mut Object = objc_retain;
                        decl.add_method(sel!(retain), f);
                        let f: extern fn(&mut Object, Sel) = objc_release;
                        decl.add_method(sel!(release), f);
                        let f: extern fn(&mut Object, Sel) = objc_dealloc;
                        decl.add_method(sel!(dealloc), f);
                    }
                    decl.register();
                });
                Class::get(stringify!($newclass)).unwrap()
            }
        }
    };
}

objc_subclass!(RRScrubber, NSScrubber, RRSCRUBBER_CLASS);
objc_subclass!(RRTouchBar, NSTouchBar, RRTOUCHBAR_CLASS);
objc_subclass!(RRCustomTouchBarItem, NSCustomTouchBarItem, RRCUSTOMITEM_CLASS);
objc_subclass!(RRPopoverTouchBarItem, NSPopoverTouchBarItem, RRPOPOVERITEM_CLASS);
objc_subclass!(RRSliderTouchBarItem, NSSliderTouchBarItem, RRSLIDER_CLASS);
