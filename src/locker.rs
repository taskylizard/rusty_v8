use std::ops::{Deref, DerefMut};

use crate::isolate::Isolate;

/// A handle to a shared isolate, allowing access to the isolate in a thread
/// safe way.
///
/// This is a recursive lock (re-entrant) and may be nested on the same thread.
#[derive(Debug)]
pub struct Locker<'a> {
  _lock: raw::Locker,
  // Hold a mutable reference to ensure exclusive access to the isolate.
  locked: &'a mut Isolate,
}

impl<'a> Locker<'a> {
  /// Claims the isolate lock.
  pub fn new(isolate: &'a mut Isolate) -> Self {
    Self {
      _lock: raw::Locker::new(isolate),
      locked: isolate,
    }
  }

  /// Returns a reference to the locked isolate.
  pub fn isolate(&self) -> &Isolate {
    self.locked
  }

  /// Returns a mutable reference to the locked isolate.
  pub fn isolate_mut(&mut self) -> &mut Isolate {
    self.locked
  }

  /// Returns if the isolate is locked by the current thread.
  pub fn is_locked(isolate: &Isolate) -> bool {
    raw::Locker::is_locked(isolate)
  }
}

impl<'a> Deref for Locker<'a> {
  type Target = Isolate;
  fn deref(&self) -> &Self::Target {
    self.isolate()
  }
}

impl<'a> DerefMut for Locker<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.isolate_mut()
  }
}

impl<'a> AsMut<Isolate> for Locker<'a> {
  fn as_mut(&mut self) -> &mut Isolate {
    self
  }
}

mod raw {
  use std::mem::MaybeUninit;

  use crate::Isolate;

  #[repr(C)]
  #[derive(Debug)]
  pub(super) struct Locker([MaybeUninit<usize>; 2]);

  impl Locker {
    pub fn new(isolate: &Isolate) -> Self {
      unsafe {
        let mut s = Self(MaybeUninit::uninit().assume_init());
        v8__Locker__CONSTRUCT(&mut s, isolate);
        // v8-locker.h disallows copying and assigning, but it does not disallow
        // moving so this is hopefully safe.
        s
      }
    }

    pub fn is_locked(isolate: &Isolate) -> bool {
      unsafe { v8__Locker__IsLocked(isolate) }
    }
  }

  impl Drop for Locker {
    fn drop(&mut self) {
      unsafe { v8__Locker__DESTRUCT(self) }
    }
  }

  unsafe extern "C" {
    fn v8__Locker__CONSTRUCT(locker: *mut Locker, isolate: *const Isolate);
    fn v8__Locker__DESTRUCT(locker: *mut Locker);
    fn v8__Locker__IsLocked(isolate: *const Isolate) -> bool;
  }
}
