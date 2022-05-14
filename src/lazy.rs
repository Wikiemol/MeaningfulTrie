use std::cell::UnsafeCell;
use std::rc::Rc;

pub type LazySupplier<T> = Rc<dyn Fn() -> T>;

pub struct Lazy<T> {
    t:  UnsafeCell<Option<T>>,
    supplier: LazySupplier<T>
}

impl<T> Lazy<T> {
    pub fn new(supplier: LazySupplier<T>) -> Lazy<T> {
        Lazy {
            t: UnsafeCell::new(None), 
            supplier
        }
    }



}

impl<T> AsRef<T> for Lazy<T> {
    fn as_ref(&self) -> &T { 
        unsafe {
            match *self.t.get() {
                None => {
                    *self.t.get() =  Some((self.supplier)());
                }
                _ => { }
            }
            (&*self.t.get()).as_ref().unwrap()
        }
    }
}

impl<T> AsMut<T> for Lazy<T> {
    fn as_mut(&mut self) -> &mut T { 
        unsafe {
            match *self.t.get() {
                None => {
                    *self.t.get() =  Some((self.supplier)());
                }
                _ => { }
            }
            (&mut *self.t.get()).as_mut().unwrap()
        }

    }
}

