use crate::isolate::Isolate;

/// Temporarily releases the isolate lock so other threads may enter.
///
/// This is not recursive; do not stack multiple Unlockers.
#[derive(Debug)]
pub struct Unlocker<'a> {
  _unlocker: raw::Unlocker,
  // Hold a mutable reference to ensure exclusive access to the isolate.
  _unlocked: &'a mut Isolate,
}

impl<'a> Unlocker<'a> {
  /// Releases the isolate lock for the lifetime of this unlocker.
  ///
  /// Callers should exit the isolate before constructing an Unlocker and
  /// re-enter after it is dropped, matching V8's C++ API usage.
  pub fn new(isolate: &'a mut Isolate) -> Self {
    Self {
      _unlocker: raw::Unlocker::new(isolate),
      _unlocked: isolate,
    }
  }
}

mod raw {
  use std::mem::MaybeUninit;

  use crate::Isolate;

  #[repr(C)]
  #[derive(Debug)]
  pub(super) struct Unlocker([MaybeUninit<usize>; 1]);

  impl Unlocker {
    pub fn new(isolate: &Isolate) -> Self {
      unsafe {
        let mut s = Self(MaybeUninit::uninit().assume_init());
        v8__Unlocker__CONSTRUCT(&mut s, isolate);
        s
      }
    }
  }

  impl Drop for Unlocker {
    fn drop(&mut self) {
      unsafe { v8__Unlocker__DESTRUCT(self) }
    }
  }

  unsafe extern "C" {
    fn v8__Unlocker__CONSTRUCT(unlocker: *mut Unlocker, isolate: *const Isolate);
    fn v8__Unlocker__DESTRUCT(unlocker: *mut Unlocker);
  }
}
