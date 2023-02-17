use iree_sys::iree::runtime::api::iree_hal_allocator_t;

#[derive(Clone)]
pub struct IreeHalAllocator {
    pub(crate) allocator_ptr: *mut iree_hal_allocator_t,
}
