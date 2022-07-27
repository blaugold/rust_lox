use crate::memory::{free_array, grow_array, grow_capacity};

pub struct Array<T> {
    count: usize,
    capacity: usize,
    elements: *mut T,
}

impl<T> Array<T> {
    pub fn new() -> Self {
        Array {
            count: 0,
            capacity: 0,
            elements: std::ptr::null_mut(),
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn elements(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.elements, self.count) }
    }

    pub fn add(&mut self, value: T) -> usize {
        if self.capacity < self.count + 1 {
            let old_capacity = self.capacity;
            self.capacity = grow_capacity(self.capacity);
            unsafe {
                self.elements = grow_array(self.elements, old_capacity, self.capacity);
            }
        }

        let offset = self.count;
        unsafe {
            self.elements.add(offset).write(value);
        }
        self.count += 1;
        offset
    }
}

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        unsafe {
            free_array(self.elements, self.capacity);
        }
    }
}
