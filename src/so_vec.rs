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
use core::convert::{AsMut, AsRef};
use core::mem::MaybeUninit;

/// `SoVec` stands for `Small optimized Vector` .
///
/// `SoVec` behaves like `std::collections::Vec`, however, it will not allocate heap memory
/// if the requested memory size is small enough.
/// Instead of heap, it uses itself as a buffer then.
///
/// To avoid allocating as much as possible, the performance is better than that of `std::collections::Vec` .
///
/// Some unsafe methods behavior is different from the same name method of `std::collections::Vec` ;
/// otherwise, (i.e. the all safe methods) behaves like that of `std::collections::Vec` .
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
    /// Creates a new empty instance.
    pub fn new(alloc: A) -> Self {
        Self::from(alloc)
    }

    /// Creates a new empty instance whose capacity is greater than or equals to `capacity` .
    ///
    /// # Panics
    ///
    /// Panics on heap memory allocation failure.
    pub fn with_capacity(capacity: usize, alloc: A) -> Self {
        let mut ret = Self::from(alloc);

        if StackBuffer::<T>::capacity() < capacity {
            unsafe {
                let heap_buffer = HeapBuffer::<T>::with_capacity(capacity, &ret.alloc);
                ret.to_heap(heap_buffer);
            }
        }

        ret
    }

    /// Returns the number of the elements `self` is holding.
    pub fn len(&self) -> usize {
        if self.is_using_stack() {
            self.as_stack().len()
        } else {
            self.as_heap().len()
        }
    }

    /// Returns true if `self` is not holding any element, or false.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

    /// Reserves the minimum capacity to insert `additional` more elements.
    ///
    /// After this method is called, `self.capacity` will return the number
    /// to be greater than or equals to `self.len() + additional` .
    ///
    /// # Panics
    ///
    /// Panics on heap allocation failure.
    pub fn reserve_exact(&mut self, additional: usize) {
        let new_capacity = self.len() + additional;

        if new_capacity <= self.capacity() {
            return;
        }

        unsafe {
            if self.is_using_stack() {
                let mut heap_buffer = HeapBuffer::<T>::with_capacity(new_capacity, &self.alloc);
                std::ptr::copy_nonoverlapping(self.as_ptr(), heap_buffer.as_mut_ptr(), self.len());
                heap_buffer.set_len(self.len());

                self.to_heap(heap_buffer);
            } else {
                let alloc = &self.alloc as *const A;
                self.as_mut_heap().set_capacity(new_capacity, &*alloc);
            }
        }
    }

    /// Appends `elm` to the end of `self` .
    ///
    /// The caller must ensure that `self` has sufficient capacity in advance.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if `self.len` is greater than or equals to
    /// `self.capacity` .
    pub unsafe fn push(&mut self, elm: T) {
        debug_assert!(self.len() < self.capacity());

        let ptr = self.as_mut_ptr().add(self.len());
        core::ptr::write(ptr, elm);
        self.set_len(self.len() + 1);
    }

    /// Removes the last element and returns it if any.
    pub fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            unsafe {
                self.set_len(self.len() - 1);
                let mut elm = MaybeUninit::uninit();
                std::ptr::copy_nonoverlapping(self.as_ptr().add(self.len()), elm.as_mut_ptr(), 1);
                Some(elm.assume_init())
            }
        }
    }

    /// Returns a raw pointer to the buffer of `self` .
    ///
    /// # Warnings
    ///
    /// The caller must ensure that `self` outlives the pointer this function returns,
    /// or else it will end up pointing to garbage. Modifying or moving `self` may cause
    /// its buffer to be reallocated, which would also make any pointers to it invalid.
    ///
    /// # Safety
    ///
    /// This method itself is safe, however, moving `self` can invalidate the returned value
    /// unlike to the same name methods of `std::collections::Vec` .
    ///
    /// `unsafe` modifier is added to alert it.
    pub unsafe fn as_ptr(&self) -> *const T {
        if self.is_using_stack() {
            self.as_stack().as_ptr()
        } else {
            self.as_heap().as_ptr()
        }
    }

    /// Returns a raw pointer to the buffer of `self` .
    ///
    /// # Warnings
    ///
    /// The caller must ensure that `self` outlives the pointer this function returns,
    /// or else it will end up pointing to garbage. Modifying or moving `self` may cause
    /// its buffer to be reallocated, which would also make any pointers to it invalid.
    ///
    /// # Safety
    ///
    /// This method itself is safe, however, moving `self` can invalidate the returned value
    /// unlike to the same name methods of `std::collections::Vec` .
    ///
    /// `unsafe` modifier is added to alert it.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        if self.is_using_stack() {
            self.as_mut_stack().as_mut_ptr()
        } else {
            self.as_mut_heap().as_mut_ptr()
        }
    }

    /// Removes the all elements keeping the allocated capacity, and set the length 0.
    ///
    /// Note this has the same effect to `self.truncate(0)` .
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Enshortens `self`, keeping the first `new_len` elements and dropping the rest.
    ///
    /// If `new_len` is greater than or equals to the current length, nothing is done.
    ///
    /// Note that this method does not have any effect on the allocated capacity of `self` .
    pub fn truncate(&mut self, new_len: usize) {
        if self.len() <= new_len {
            return;
        }

        unsafe {
            for i in new_len..self.len() {
                core::ptr::drop_in_place(self.as_mut_ptr().add(i));
            }

            self.set_len(new_len);
        }
    }

    /// Shrinks the capacitance of `self` as much as possible.
    pub fn shrink_to_fit(&mut self) {
        if self.is_using_stack() {
            return;
        } else {
            let alloc = &self.alloc as *const A;
            let new_capacity = self.len();
            unsafe { self.as_mut_heap().set_capacity(new_capacity, &*alloc) };
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

    /// Disables small optimization forces to overwrite `self.stack` .
    ///
    /// Note that this method does not move each element.
    unsafe fn to_heap(&mut self, new_buffer: HeapBuffer<T>) {
        debug_assert!(self.is_using_stack());

        let ptr = &mut self.buffer as *mut StackBuffer<T>;
        let ptr = ptr as *mut u8;
        let ptr = ptr as *mut HeapBuffer<T>;

        core::ptr::write(ptr, new_buffer);
        self.as_mut_stack().disable();
    }
}

impl<T, A> From<A> for SoVec<T, A>
where
    A: GlobalAlloc,
{
    fn from(alloc: A) -> Self {
        Self {
            buffer: StackBuffer::<T>::new(),
            alloc,
        }
    }
}

impl<T, A> Default for SoVec<T, A>
where
    A: GlobalAlloc + Default,
{
    fn default() -> Self {
        Self::from(A::default())
    }
}

impl<T, A> AsRef<[T]> for SoVec<T, A>
where
    A: GlobalAlloc,
{
    fn as_ref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.len()) }
    }
}

impl<T, A> AsMut<[T]> for SoVec<T, A>
where
    A: GlobalAlloc,
{
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }
}

impl<T, A> Drop for SoVec<T, A>
where
    A: GlobalAlloc,
{
    fn drop(&mut self) {
        self.clear();

        if !self.is_using_stack() {
            let alloc = &self.alloc as *const A;
            unsafe { self.as_mut_heap().pre_drop(&*alloc) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::TestAllocator;

    #[test]
    fn from() {
        {
            let alloc = TestAllocator::new();
            let v = SoVec::<u8, TestAllocator>::from(alloc);

            assert_eq!(0, v.len());
        }

        {
            let alloc = TestAllocator::new();
            let v = SoVec::<String, TestAllocator>::from(alloc);

            assert_eq!(0, v.len());
        }

        {
            let alloc = TestAllocator::new();
            let v = SoVec::<[u8; 3], TestAllocator>::from(alloc);

            assert_eq!(0, v.len());
        }
    }

    #[test]
    fn with_capacity() {
        for i in 0..(StackBuffer::<u8>::capacity() + 10) {
            let alloc = TestAllocator::new();
            let v = SoVec::<u8, TestAllocator>::with_capacity(i, alloc);

            assert_eq!(0, v.len());
            assert!(i <= v.capacity());
        }

        for i in 0..(StackBuffer::<String>::capacity() + 10) {
            let alloc = TestAllocator::new();
            let v = SoVec::<String, TestAllocator>::with_capacity(i, alloc);

            assert_eq!(0, v.len());
            assert!(i <= v.capacity());
        }
    }

    #[test]
    fn reserve_exact() {
        for i in 0..(StackBuffer::<u8>::capacity() + 10) {
            let alloc = TestAllocator::new();
            let mut v = SoVec::<u8, TestAllocator>::with_capacity(i, alloc);

            for j in 0..(StackBuffer::<u8>::capacity() + 10) {
                v.reserve_exact(j);
                assert_eq!(0, v.len());
                assert!(i <= v.capacity());
                assert!(j <= v.capacity());
            }
        }

        for i in 0..(StackBuffer::<String>::capacity() + 10) {
            let alloc = TestAllocator::new();
            let mut v = SoVec::<String, TestAllocator>::with_capacity(i, alloc);

            for j in 0..(StackBuffer::<String>::capacity() + 10) {
                v.reserve_exact(j);
                assert_eq!(0, v.len());
                assert!(i <= v.capacity());
                assert!(j <= v.capacity());
            }
        }
    }

    #[test]
    fn push() {
        {
            let origin: Vec<u8> = (0..=u8::MAX).collect();

            let mut v = SoVec::<u8, TestAllocator>::default();
            let init_capacity = v.capacity();

            for i in 0..init_capacity {
                unsafe { v.push(i as u8) };
                assert_eq!(&origin[0..=i], v.as_ref());
            }

            for i in init_capacity..=(u8::MAX as usize) {
                unsafe {
                    v.reserve_exact(1);
                    v.push(i as u8);
                }
                assert_eq!(&origin[0..=i], v.as_ref());
            }
        }
        {
            let origin: Vec<String> = (0..=u8::MAX).map(|i| i.to_string()).collect();

            let mut v = SoVec::<String, TestAllocator>::default();
            let init_capacity = v.capacity();

            for i in 0..init_capacity {
                unsafe { v.push(i.to_string()) };
                assert_eq!(&origin[0..=i], v.as_ref());
            }

            for i in init_capacity..=3 * init_capacity {
                unsafe {
                    v.reserve_exact(1);
                    v.push(i.to_string());
                }
                assert_eq!(&origin[0..=i], v.as_ref());
            }
        }
    }

    #[test]
    fn pop() {
        {
            let alloc = TestAllocator::new();
            let mut v = SoVec::<u8, TestAllocator>::with_capacity((u8::MAX as usize) + 1, alloc);
            assert_eq!(None, v.pop());

            for i in 0..=u8::MAX {
                unsafe {
                    v.push(i);
                }
            }

            for i in (0..=u8::MAX).rev() {
                assert_eq!(Some(i), v.pop());
            }

            assert_eq!(None, v.pop());
        }

        {
            let alloc = TestAllocator::new();
            let mut v =
                SoVec::<String, TestAllocator>::with_capacity((u8::MAX as usize) + 1, alloc);
            assert_eq!(None, v.pop());

            for i in 0..=u8::MAX {
                unsafe {
                    v.push(i.to_string());
                }
            }

            for i in (0..=u8::MAX).rev() {
                assert_eq!(Some(i.to_string()), v.pop());
            }

            assert_eq!(None, v.pop());
        }
    }
}
