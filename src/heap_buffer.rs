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

use core::alloc::Layout;
use core::mem::{align_of, size_of};

pub struct HeapBuffer<T> {
    ptr: *mut T,
    len_: usize,
    cap_: usize,
}

impl<T> HeapBuffer<T> {
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
