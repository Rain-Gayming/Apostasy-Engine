use downcast_rs::{Downcast, impl_downcast};

pub trait Component: Downcast {}
impl_downcast!(Component);

impl<T: 'static> Component for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn new_empty_column(&self) -> Box<dyn Component> {
        Box::new(Vec::<T>::new())
    }
}
