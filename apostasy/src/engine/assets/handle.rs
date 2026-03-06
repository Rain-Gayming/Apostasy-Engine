use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Type errased handled, used for storing handles of mixed types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UntypedHandle {
    pub id: u64,
    pub type_name: &'static str,
}

/// A handle to an asset
pub struct Handle<T> {
    pub id: u64,
    phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    /// Create a new handle
    pub fn new() -> Handle<T> {
        Handle {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            phantom: PhantomData,
        }
    }

    /// Create a new handle with a specific id
    pub fn with_id(id: u64) -> Handle<T> {
        Handle {
            id: id,
            phantom: PhantomData,
        }
    }

    /// Get the raw id of this handle
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn untyped(&self) -> UntypedHandle {
        UntypedHandle {
            id: self.id,
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Handle<T> {
        Handle {
            id: self.id,
            phantom: PhantomData,
        }
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Handle<T>) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Handle {{ id: {} }}", self.id)
    }
}

impl<T> fmt::Display for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
