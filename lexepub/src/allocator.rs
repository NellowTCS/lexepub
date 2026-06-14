use core::alloc::{GlobalAlloc, Layout};

extern "C" {
    fn memalign(alignment: usize, size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
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

#[export_name = "_RNvCs8KYW7hILzp0_7___rustc35___rust_no_alloc_shim_is_unstable_v2"]
fn __rust_no_alloc_shim_is_unstable_v2() {}
