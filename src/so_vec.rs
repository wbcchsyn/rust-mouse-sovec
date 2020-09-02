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
    /// Returns true if `self` is using StackBuffer; otherwise, i.e. `self` is using `HeapBuffer`,
    /// returns false.
    fn is_using_stack(&self) -> bool {
        self.buffer.is_available()
    }
}
