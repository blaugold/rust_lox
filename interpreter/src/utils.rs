use std::cell::UnsafeCell;

pub struct Late<T> {
    value: UnsafeCell<Option<T>>,
}

impl<T> Late<T> {
    pub fn new() -> Late<T> {
        Late {
            value: UnsafeCell::new(None),
        }
    }

    pub fn set(&self, value: T) {
        unsafe {
            let inner = self.value.get();
            match *inner {
                Some(_) => panic!(),
                None => *inner = Some(value),
            }
        }
    }

    pub fn get(&self) -> Option<&T> {
        unsafe {
            // SAFETY: The only time a mutable reference is taken is in set.
            // If set was called there will be a value. Set cannot be called a
            // second time without panicking. Therefor it is save to pass out
            // immutable references after value has been set.
            match &*self.value.get() {
                None => None,
                Some(value) => Some(&value),
            }
        }
    }
}
