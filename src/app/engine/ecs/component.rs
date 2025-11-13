use downcast_rs::{Downcast, impl_downcast};
use std::any::TypeId;

pub trait Component: Downcast {
    fn type_id_dyn(&self) -> TypeId;
}

impl<T: 'static> Component for Vec<T> {
    fn type_id_dyn(&self) -> TypeId {
        TypeId::of::<Vec<T>>()
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
