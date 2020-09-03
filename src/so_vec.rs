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

use crate::heap_buffer::HeapBuffer;
use crate::stack_buffer::StackBuffer;
use core::alloc::GlobalAlloc;

/// `SoVec` stands for `Small optimized Vector` .
///
/// `SoVec` behaves like `std::collections::Vec`, however, it will not allocate heap memory
/// if the requested memory size is small enough.
/// Instead of heap, it uses itself as a buffer then.
///
/// To avoid allocating as much as possible, the performance is better than that of `std::collections::Vec` .
pub struct SoVec<T, A>
where
    A: GlobalAlloc,
{
    buffer: StackBuffer<T>,
    alloc: A,
}

impl<T, A> SoVec<T, A>
where
    A: GlobalAlloc,
{
    /// Returns the number of the elements `self` is holding.
    pub fn len(&self) -> usize {
        if self.is_using_stack() {
            self.as_stack().len()
        } else {
            self.as_heap().len()
        }
    }

    /// Forces the length of `self` to `new\_len` .
    ///
    /// # Safety
    ///
    /// - `new\_len` must be less than or equal to `capacity` .
    /// - The elements at old_len..new\_len must be initialized when extending.
    /// - The elements at new_len..old\_len must be dropped when shrinking.
    pub unsafe fn set_len(&mut self, new_len: usize) {
        if self.is_using_stack() {
            self.as_mut_stack().set_len(new_len);
        } else {
            self.as_mut_heap().set_len(new_len);
        }
    }

    /// Returns the number of the elements `self` can hold without allocating.
    pub fn capacity(&self) -> usize {
        if self.is_using_stack() {
            StackBuffer::<T>::capacity()
        } else {
            self.as_heap().capacity()
        }
    }

    /// Returns a raw pointer to the buffer of `self` .
    ///
    /// The caller must ensure that `self` outlives the pointer this function returns,
    /// or else it will end up pointing to garbage. Modifying or moving `self` may cause
    /// its buffer to be reallocated, which would also make any pointers to it invalid.
    pub fn as_ptr(&self) -> *const T {
        if self.is_using_stack() {
            self.as_stack().as_ptr()
        } else {
            self.as_heap().as_ptr()
        }
    }

    /// Returns a raw pointer to the buffer of `self` .
    ///
    /// The caller must ensure that `self` outlives the pointer this function returns,
    /// or else it will end up pointing to garbage. Modifying or moving `self` may cause
    /// its buffer to be reallocated, which would also make any pointers to it invalid.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        if self.is_using_stack() {
            self.as_mut_stack().as_mut_ptr()
        } else {
            self.as_mut_heap().as_mut_ptr()
        }
    }

    /// Returns true if `self` is using StackBuffer; otherwise, i.e. `self` is using `HeapBuffer`,
    /// returns false.
    fn is_using_stack(&self) -> bool {
        self.buffer.is_available()
    }

    /// Returns `self.bufer` .
    fn as_stack(&self) -> &StackBuffer<T> {
        debug_assert!(self.is_using_stack());
        &self.buffer
    }

    /// Returns `self.bufer` .
    fn as_mut_stack(&mut self) -> &mut StackBuffer<T> {
        debug_assert!(self.is_using_stack());
        &mut self.buffer
    }

    /// Forces to regards `self.buffer` as HeapBuffer and returns it.
    fn as_heap(&self) -> &HeapBuffer<T> {
        debug_assert_eq!(false, self.is_using_stack());
        let ptr = &self.buffer as *const StackBuffer<T>;
        let ptr = ptr as *const u8;
        let ptr = ptr as *const HeapBuffer<T>;
        unsafe { &*ptr }
    }

    /// Forces to regards `self.buffer` as HeapBuffer and returns it.
    fn as_mut_heap(&mut self) -> &mut HeapBuffer<T> {
        debug_assert_eq!(false, self.is_using_stack());
        let ptr = &mut self.buffer as *mut StackBuffer<T>;
        let ptr = ptr as *mut u8;
        let ptr = ptr as *mut HeapBuffer<T>;
        unsafe { &mut *ptr }
    }
}
