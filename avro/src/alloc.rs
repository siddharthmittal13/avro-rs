// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Optional custom allocator support.
//!
//! This module is only available when the `custom_allocator` feature is enabled.
//! It is **opt-in**: nothing here changes how `apache-avro` allocates unless a
//! downstream program explicitly installs [`CustomAllocator`] as its
//! `#[global_allocator]`.
//!
//! [`CustomAllocator`] is a minimal [`GlobalAlloc`] that delegates every
//! operation to the [`System`] allocator. It works as a drop-in global
//! allocator and as a base you can extend to add custom behavior (logging,
//! pooling, metrics, ...) without reimplementing the platform allocation logic.
//!
//! # Example
//!
//! ```no_run
//! use apache_avro::alloc::CustomAllocator;
//!
//! #[global_allocator]
//! static GLOBAL: CustomAllocator = CustomAllocator;
//! ```

use std::alloc::{GlobalAlloc, Layout, System};

/// A minimal custom [`GlobalAlloc`] that delegates every operation to the
/// [`System`] allocator.
///
/// Install it as a `#[global_allocator]` directly, or use it as a starting
/// point for a custom allocator without having to reimplement the platform
/// allocation logic. Its behavior is identical to the default system allocator.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CustomAllocator;

// SAFETY: Every method simply forwards to `System`, which is a correct
// `GlobalAlloc` implementation, preserving all of its invariants.
unsafe impl GlobalAlloc for CustomAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Upheld by the caller of the `GlobalAlloc` method.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: Upheld by the caller of the `GlobalAlloc` method.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Upheld by the caller of the `GlobalAlloc` method.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: Upheld by the caller of the `GlobalAlloc` method.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_allocator_delegates_to_system() {
        let allocator = CustomAllocator;
        let layout = Layout::from_size_align(48, 8).unwrap();

        // SAFETY: The block is allocated and freed through the same allocator
        // with the same layout.
        unsafe {
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());
            ptr.write_bytes(0xAB, layout.size());
            allocator.dealloc(ptr, layout);
        }
    }

    #[test]
    fn test_custom_allocator_realloc() {
        let allocator = CustomAllocator;
        let layout = Layout::from_size_align(16, 8).unwrap();

        // SAFETY: `ptr` is allocated, grown and freed with matching layouts.
        unsafe {
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());
            *ptr = 42;
            let grown = allocator.realloc(ptr, layout, 128);
            assert!(!grown.is_null());
            assert_eq!(*grown, 42);
            let grown_layout = Layout::from_size_align(128, 8).unwrap();
            allocator.dealloc(grown, grown_layout);
        }
    }
}
