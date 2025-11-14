use downcast_rs::{Downcast, impl_downcast};
use std::{any::TypeId, fmt::Debug};

pub trait Component: Downcast {
    fn type_id_dyn(&self) -> TypeId;

    fn clone_box(&self) -> Box<dyn Component>;
}

impl<T: 'static + Clone> Component for Vec<T> {
    fn type_id_dyn(&self) -> TypeId {
        TypeId::of::<Vec<T>>()
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Component> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl_downcast!(Component);

impl PartialEq for dyn Component {
    fn eq(&self, other: &Self) -> bool {
        if self.type_id_dyn() != other.type_id_dyn() {
            return false;
        }
        self.type_id_dyn() == other.type_id_dyn()
    }
}

impl Debug for dyn Component {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("type")
            .field("type_id", &::std::any::TypeId::of::<Self>())
            .finish()
    }
}
