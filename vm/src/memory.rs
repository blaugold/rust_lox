use std::{
    alloc::{self, Layout},
    process::exit,
};

pub fn grow_capacity(capacity: usize) -> usize {
    if capacity < 8 {
        8
    } else {
        capacity * 2
    }
}

pub unsafe fn grow_array<T>(ptr: *mut T, old_size: usize, new_size: usize) -> *mut T {
    reallocate(ptr, old_size, new_size)
}

pub unsafe fn free_array<T>(ptr: *mut T, old_size: usize) {
    reallocate(ptr, old_size, 0);
}

unsafe fn reallocate<T>(ptr: *mut T, old_size: usize, new_size: usize) -> *mut T {
    let old_layout = Layout::array::<T>(old_size).unwrap();

    if new_size == 0 {
        alloc::dealloc(ptr as *mut u8, old_layout);
        return std::ptr::null_mut();
    }

    let new_layout = Layout::array::<T>(new_size).unwrap();
    let result = alloc::realloc(ptr as *mut u8, old_layout, new_layout.size());
    if result.is_null() {
        exit(1)
    }
    result as *mut T
}
