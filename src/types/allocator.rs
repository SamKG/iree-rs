use iree_sys::{self, iree::runtime::api::iree_allocator_t};

pub struct IreeAllocator {
    pub(crate) allocator: iree_allocator_t,
}

impl IreeAllocator {
    /// Creates a default allocator that uses the system allocator (typically malloc).
    pub fn system_allocator() -> Self {
        // FIXME: This emulates the functionality of the `iree_system_allocator` macro. We should ideally be able to use that macro directly.
        Self {
            allocator: iree_allocator_t {
                self_: std::ptr::null_mut(),
                ctl: Some(iree_sys::iree::runtime::api::iree_allocator_system_ctl as _),
            },
        }
    }
}
