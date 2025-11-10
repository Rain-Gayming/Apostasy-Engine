use downcast_rs::{Downcast, impl_downcast};

pub trait Component: Downcast {}
impl_downcast!(Component);
