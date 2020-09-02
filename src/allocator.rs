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

use core::sync::atomic::{AtomicI64, Ordering};
use std::alloc::{GlobalAlloc, Layout, System};

/// Wrappter of `std::alloc::System` .
/// It counts allocation and deallocation, and check the both
/// numbers are same on drop.
pub struct TestAllocator {
    count: AtomicI64,
}

unsafe impl GlobalAlloc for TestAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let system = System;
        let ptr = system.alloc(layout);

        if !ptr.is_null() {
            self.count.fetch_add(1, Ordering::Acquire);
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        assert_eq!(false, ptr.is_null());

        let system = System;
        let c = self.count.fetch_sub(1, Ordering::Release);

        if c <= 0 {
            panic!("Calls dealloc() too many times");
        }

        system.dealloc(ptr, layout);
    }
}

impl Drop for TestAllocator {
    fn drop(&mut self) {
        if self.count.load(Ordering::Relaxed) != 0 {
            panic!("Memory Leak!");
        }
    }
}
