//! ESP-IDF heap allocator shim.
//!
//! Maps Rust's `#[global_allocator]` interface onto `memalign`/`free`/`realloc`
//! from ESP-IDF's heap API.  Only compiled for `target_os = "espidf"` (see
//! `lib.rs`).
//!
//! The `__rust_no_alloc_shim_is_unstable` symbol is required by the Rust
//! compiler on xtensa-esp32s3-espidf targets when a custom allocator is
//! registered, if omitted you get a linker error about a missing allocator
//! shim.

use core::alloc::{GlobalAlloc, Layout};

extern "C" {
    fn memalign(alignment: usize, size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    /// ESP-IDF `realloc` follows standard semantics: if `ptr` is null it
    /// behaves like `malloc`, and if `size` is zero it behaves like `free`.
    fn realloc(ptr: *mut u8, size: usize) -> *mut u8;
}

struct EspAlloc;

unsafe impl GlobalAlloc for EspAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        memalign(layout.align(), layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr)
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        realloc(ptr, new_size)
    }
}

#[global_allocator]
static ALLOCATOR: EspAlloc = EspAlloc;

// Required by the Rust compiler on xtensa-espidf targets when a global
// allocator is set.
#[no_mangle]
pub extern "C" fn __rust_no_alloc_shim_is_unstable() {}
