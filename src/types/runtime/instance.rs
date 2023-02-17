use iree_sys::{
    helper::IREE_CHECK_OK,
    iree::runtime::api::{
        iree_hal_device_t, iree_runtime_instance_create, iree_runtime_instance_host_allocator,
        iree_runtime_instance_options_initialize, iree_runtime_instance_options_t,
        iree_runtime_instance_options_use_all_available_drivers, iree_runtime_instance_release,
        iree_runtime_instance_t, iree_runtime_instance_try_create_default_device,
        iree_string_view_t,
    },
};

use crate::{
    err::IreeError,
    types::{allocator::IreeAllocator, hal_device::IreeHalDevice, status::IreeStatus},
};

pub struct IreeRuntimeInstanceOptions {
    options: iree_runtime_instance_options_t,
}

pub struct IreeRuntimeInstanceOptionsBuilder {
    options: iree_runtime_instance_options_t,
}

impl Default for IreeRuntimeInstanceOptionsBuilder {
    fn default() -> Self {
        let mut options = iree_runtime_instance_options_t::default();
        unsafe {
            iree_runtime_instance_options_initialize(&mut options);
        }
        Self { options }
    }
}

impl IreeRuntimeInstanceOptionsBuilder {
    pub fn use_all_available_drivers(&mut self) -> &mut Self {
        unsafe {
            iree_runtime_instance_options_use_all_available_drivers(&mut self.options);
        }
        self
    }
    pub fn build(&self) -> IreeRuntimeInstanceOptions {
        IreeRuntimeInstanceOptions {
            options: self.options,
        }
    }
}

pub struct IreeRuntimeInstance {
    pub(crate) instance_ptr: *mut iree_runtime_instance_t,
}

impl IreeRuntimeInstance {
    pub fn try_from_options(
        options: &IreeRuntimeInstanceOptions,
        allocator: &IreeAllocator,
    ) -> Result<Self, IreeError> {
        let mut instance_ptr = std::mem::MaybeUninit::<*mut iree_runtime_instance_t>::uninit();
        unsafe {
            let status = iree_runtime_instance_create(
                &options.options,
                allocator.allocator,
                instance_ptr.as_mut_ptr(),
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(IreeStatus { status }, allocator));
            }
        }
        Ok(Self {
            instance_ptr: unsafe { instance_ptr.assume_init() },
        })
    }

    pub fn host_allocator(&self) -> IreeAllocator {
        let allocator = unsafe { iree_runtime_instance_host_allocator(self.instance_ptr) };
        IreeAllocator { allocator }
    }

    pub fn try_create_default_device(&self, driver_name: &str) -> Result<IreeHalDevice, IreeError> {
        let driver_name = iree_string_view_t {
            data: driver_name.as_ptr() as _,
            size: driver_name.len() as _,
        };
        let mut device_ptr = std::mem::MaybeUninit::<*mut iree_hal_device_t>::uninit();
        unsafe {
            let status = iree_runtime_instance_try_create_default_device(
                self.instance_ptr,
                driver_name,
                device_ptr.as_mut_ptr(),
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &self.host_allocator(),
                ));
            }
        }
        Ok(IreeHalDevice {
            device_ptr: unsafe { device_ptr.assume_init() },
        })
    }
}

impl Drop for IreeRuntimeInstance {
    fn drop(&mut self) {
        unsafe {
            iree_runtime_instance_release(self.instance_ptr);
        }
    }
}
