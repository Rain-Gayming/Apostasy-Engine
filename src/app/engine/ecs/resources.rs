use downcast_rs::{Downcast, impl_downcast};

pub trait Resource: Downcast {}
impl_downcast!(Resource);
