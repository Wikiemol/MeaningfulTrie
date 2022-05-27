use std::cell::UnsafeCell;
use std::rc::Rc;

pub type LazyFunction<V, T> = fn(value: V) -> T;
pub type LazySupplier<T> = Rc<dyn Fn() -> T>;


#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum LazyValue<'a, V, T> {
    Function(V, LazyFunction<&'a V, T>),
    Value(T)
}

pub struct DerivedLazy<'a, V, T> {
    value: UnsafeCell<LazyValue<'a, V, T>>
}

impl<'a, V, T: 'a> DerivedLazy<'a, V, T> {
    pub fn new(input: V, supplier: LazyFunction<&'a V, T>) -> DerivedLazy<V, T> {
        DerivedLazy {
            value: UnsafeCell::new(LazyValue::Function(input, supplier))
        }
    }

    pub fn get(&self) -> &T {
        unsafe {
            match &*self.value.get() {
                LazyValue::Function(input, function) => {
                    *self.value.get() = LazyValue::Value((function)(&input));
                    match &*self.value.get() {
                        LazyValue::Value(v) => &v,
                        _ => panic!("Invalid state")
                    }
                }
                LazyValue::Value(value) => {
                    &value
                }

            }
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            match &mut *self.value.get() {
                LazyValue::Function(input, function) => {
                    *self.value.get() = LazyValue::Value((function)(input));
                    match &mut *self.value.get() {
                        LazyValue::Value(v) => v,
                        _ => panic!("Invalid state")
                    }
                }
                LazyValue::Value(value) => {
                    value
                }

            }
        }
    }
}




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

