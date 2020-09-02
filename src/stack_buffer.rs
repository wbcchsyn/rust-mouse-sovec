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
use core::marker::PhantomData;
use core::mem::size_of;

type Buffer0 = usize;
type Buffer1 = [u8; size_of::<HeapBuffer<u8>>() - 1];
type Len = u8;

#[repr(C)]
pub struct StackBuffer<T> {
    _buf0: Buffer0,
    _buf1: Buffer1,
    len_: Len,
    _marker: PhantomData<T>,
}

impl<T> StackBuffer<T> {
    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        debug_assert!(self.is_available());
        self.len_ as usize
    }

    /// Forces the length of `self` to `new\_len` .
    ///
    /// # Safety
    ///
    /// - `new\_len` must be less than or equal to `capacity` .
    /// - The elements at old_len..new\_len must be initialized when extending.
    /// - The elements at new_len..old\_len must be dropped when shrinking.
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(self.is_available());
        debug_assert!(new_len <= Self::capacity());
        debug_assert!(new_len <= u8::MAX as usize);
        self.len_ = new_len as u8;
    }

    /// Returns the max number of the elements `StackBuffer` can hold.
    pub const fn capacity() -> usize {
        (size_of::<Buffer0>() + size_of::<Buffer1>()) / size_of::<T>()
    }

    /// Returns a raw pointer to the buffer.
    pub fn as_ptr(&self) -> *const T {
        debug_assert!(self.is_available());
        let ptr = &self._buf0 as *const usize;
        ptr as *const T
    }

    /// Returns a raw pointer to the buffer.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        debug_assert!(self.is_available());
        let ptr = &mut self._buf0 as *mut usize;
        ptr as *mut T
    }

    /// Returns true if `self` can be used, or false.
    pub fn is_available(&self) -> bool {
        self.len_ != u8::MAX
    }

    /// Disable to use `self` .
    pub fn disable(&mut self) {
        self.len_ = u8::MAX;
    }
}

#[cfg(test)]
impl<T> Drop for StackBuffer<T> {
    fn drop(&mut self) {
        if self.is_available() {
            assert_eq!(0, self.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::align_of;

    #[test]
    fn size() {
        assert_eq!(
            size_of::<HeapBuffer<u8>>() + size_of::<usize>(),
            size_of::<StackBuffer<u8>>()
        );
        assert_eq!(
            size_of::<HeapBuffer<usize>>() + size_of::<usize>(),
            size_of::<StackBuffer<usize>>()
        );
    }

    #[test]
    fn align() {
        assert!(align_of::<HeapBuffer<u8>>() <= align_of::<StackBuffer<u8>>());
        assert!(align_of::<HeapBuffer<usize>>() <= align_of::<StackBuffer<usize>>());
    }

    #[test]
    fn capacity() {
        let buffer_size = size_of::<StackBuffer<u8>>() - size_of::<Len>();

        assert_eq!(buffer_size / size_of::<u8>(), StackBuffer::<u8>::capacity());

        assert_eq!(
            buffer_size / size_of::<usize>(),
            StackBuffer::<usize>::capacity()
        );

        type Foo = [u8; 3];
        assert_eq!(
            buffer_size / size_of::<Foo>(),
            StackBuffer::<Foo>::capacity()
        );

        type Bar = [u8; 1024];
        assert_eq!(
            buffer_size / size_of::<Bar>(),
            StackBuffer::<Bar>::capacity()
        );
    }
}
