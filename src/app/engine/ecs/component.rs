use downcast_rs::{Downcast, impl_downcast};

pub trait Component: Downcast {}
impl<T: 'static> Component for Vec<T> {}
impl_downcast!(Component);
