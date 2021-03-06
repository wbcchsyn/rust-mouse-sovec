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
use std::alloc::handle_alloc_error;

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
    pub unsafe fn with_capacity<A>(capacity: usize, alloc: &A) -> Self
    where
        A: GlobalAlloc,
    {
        debug_assert_ne!(0, capacity);

        let size = capacity
            .checked_mul(size_of::<T>())
            .expect("Allocating memory size is too large.");
        let align = align_of::<T>();
        let layout = Layout::from_size_align(size, align).unwrap_or_else(|e| panic!("{}", e));

        let ptr = alloc.alloc(layout) as *mut T;
        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        Self {
            ptr,
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

    /// Reallocates the heap and forces the capacity of `self` to `new\_capacity` .
    ///
    /// # Safety
    ///
    /// - The behavior is undefined if `new\_capacity` is 0.
    /// - `new_capacity` must be greater than or equals to `len` .
    pub unsafe fn set_capacity<A>(&mut self, new_capacity: usize, alloc: &A)
    where
        A: GlobalAlloc,
    {
        debug_assert_ne!(0, new_capacity);
        debug_assert!(self.len() <= new_capacity);

        let layout = self.layout();
        let new_size = new_capacity
            .checked_mul(size_of::<T>())
            .expect("Allocating memory size is too large.");
        let ptr = alloc.realloc(self.ptr as *mut u8, layout, new_size) as *mut T;

        if ptr.is_null() {
            let layout = Layout::from_size_align(new_size, layout.align())
                .unwrap_or_else(|e| panic!("{}", e));
            handle_alloc_error(layout);
        } else {
            self.ptr = ptr;
            self.cap_ = new_capacity;
        }
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

    #[test]
    fn set_capacity() {
        for i in 1..10 {
            let alloc = TestAllocator::new();
            let mut b = unsafe { HeapBuffer::<String>::with_capacity(i, &alloc) };

            for j in 1..10 {
                unsafe { b.set_capacity(j, &alloc) };
                assert_eq!(0, b.len());
                assert!(j <= b.capacity());
            }

            b.pre_drop(&alloc);
        }
    }
}
