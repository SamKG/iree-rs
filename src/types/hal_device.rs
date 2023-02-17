use iree_sys::iree::runtime::api::{iree_hal_device_release, iree_hal_device_t};

pub struct IreeHalDevice {
    pub(crate) device_ptr: *mut iree_hal_device_t,
}

impl Drop for IreeHalDevice {
    fn drop(&mut self) {
        unsafe {
            iree_hal_device_release(self.device_ptr);
        }
    }
}
