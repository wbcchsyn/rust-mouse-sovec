// Copyright 2020 Shin Yoshida
//
// "LGPL-3.0-or-later OR Apache-2.0"
//
// This is part of mouse-sovec
//
//  mouse-sovec is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Lesser General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  mouse-sovec is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Lesser General Public License for more details.
//
//  You should have received a copy of the GNU Lesser General Public License
//  along with mouse-sovec.  If not, see <http://www.gnu.org/licenses/>.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::alloc::{GlobalAlloc, Layout};
use core::mem::{align_of, size_of};

pub struct HeapBuffer<T> {
    ptr: *mut T,
    len_: usize,
    cap_: usize,
}

impl<T> HeapBuffer<T> {
    /// Allocates heap memory using `alloc` and creates a new instance whose capacity is greater than or
    /// equals to `capacity` .
    ///
    /// # Safety
    ///
    /// `capacity` must not be 0.
    ///
    /// # Panics
    ///
    /// Panics if failed to allocate heap memory.
    pub unsafe fn with_capacity<A>(capacity: usize, alloc: &A) -> Self
    where
        A: GlobalAlloc,
    {
        debug_assert_ne!(0, capacity);

        let size = capacity * size_of::<T>();
        let align = align_of::<T>();
        let layout = Layout::from_size_align(size, align).expect(alloc_error_message());

        let ptr = unsafe { alloc.alloc(layout) as *mut T };
        Self {
            ptr: check_alloc(ptr),
            len_: 0,
            cap_: capacity,
        }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.len_
    }

    /// Forces the length of `self` to `new\_len` .
    ///
    /// # Safety
    ///
    /// - `new\_len` must be less than or equal to `capacity` .
    /// - The elements at old_len..new\_len must be initialized when extending.
    /// - The elements at new_len..old\_len must be dropped when shrinking.
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());
        self.len_ = new_len;
    }

    /// Returns the number of elements which `self` can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.cap_
    }

    /// Returns a raw pointer to the buffer.
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Returns a raw pointer to the buffer.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }

    /// Deallocates the owing heap buffer.
    ///
    /// `pre_drop` must be called before dropped and `self` must not be used after that.
    ///
    /// Because `HeapBuffer` does not have the allocator, Drop implementation can't do this.
    ///
    /// # Warnings
    ///
    /// This method do just deallocation.
    /// The elements must be dropped in advance.
    #[cfg(not(test))]
    pub fn pre_drop<A>(&mut self, alloc: &A)
    where
        A: GlobalAlloc,
    {
        unsafe {
            alloc.dealloc(self.as_mut_ptr() as *mut u8, self.layout());
        }
    }

    /// Deallocates the owing heap buffer.
    ///
    /// `pre_drop` must be called before dropped and `self` must not be used after that.
    ///
    /// Because `HeapBuffer` does not have the allocator, Drop implementation can't do this.
    ///
    /// # Warnings
    ///
    /// This method do just deallocation.
    /// The elements must be dropped in advance.
    #[cfg(test)]
    pub fn pre_drop<A>(&mut self, alloc: &A)
    where
        A: GlobalAlloc,
    {
        assert_eq!(0, self.len());
        assert_eq!(false, self.as_ptr().is_null());

        unsafe {
            alloc.dealloc(self.as_mut_ptr() as *mut u8, self.layout());
        }

        self.ptr = core::ptr::null_mut();
    }

    /// Returns the layout allocating the heap.
    fn layout(&self) -> Layout {
        let size = size_of::<T>() * self.capacity();
        let align = align_of::<T>();

        unsafe { Layout::from_size_align_unchecked(size, align) }
    }
}

#[cfg(test)]
impl<T> Drop for HeapBuffer<T> {
    fn drop(&mut self) {
        assert!(self.ptr.is_null());
    }
}

fn check_alloc<T>(ptr: *mut T) -> *mut T {
    if ptr.is_null() {
        panic!(alloc_error_message());
    }

    ptr
}

fn alloc_error_message() -> &'static str {
    "Failed to allocate heap memory."
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::TestAllocator;

    #[test]
    fn constructor() {
        for i in 1..10 {
            let alloc = TestAllocator::new();
            let mut b = unsafe { HeapBuffer::<String>::with_capacity(i, &alloc) };

            assert_eq!(0, b.len());
            assert!(i <= b.capacity());

            b.pre_drop(&alloc);
        }

        for i in 1..10 {
            let alloc = TestAllocator::new();
            let mut b = unsafe { HeapBuffer::<u8>::with_capacity(i, &alloc) };

            assert_eq!(0, b.len());
            assert!(i <= b.capacity());

            b.pre_drop(&alloc);
        }
    }
}
