use iree_sys::{
    helper::IREE_CHECK_OK,
    iree::runtime::api::{
        iree_const_byte_span_t, iree_runtime_call_initialize_by_name, iree_runtime_call_t,
        iree_runtime_session_append_bytecode_module_from_memory,
        iree_runtime_session_create_with_device, iree_runtime_session_device_allocator,
        iree_runtime_session_options_initialize, iree_runtime_session_options_t,
        iree_runtime_session_release, iree_runtime_session_t, iree_string_view_t,
    },
};

use crate::{
    err::IreeError,
    types::{
        allocator::IreeAllocator, hal_allocator::IreeHalAllocator, hal_device::IreeHalDevice,
        status::IreeStatus,
    },
};

use super::{call::IreeRuntimeCall, instance::IreeRuntimeInstance};

pub struct IreeRuntimeSessionOptions {
    options: iree_runtime_session_options_t,
}

pub struct IreeRuntimeSessionOptionsBuilder {
    options: iree_runtime_session_options_t,
}

impl Default for IreeRuntimeSessionOptionsBuilder {
    fn default() -> Self {
        let mut options = iree_runtime_session_options_t::default();
        unsafe {
            iree_runtime_session_options_initialize(&mut options);
        }
        Self { options }
    }
}

impl IreeRuntimeSessionOptionsBuilder {
    pub fn build(&self) -> IreeRuntimeSessionOptions {
        IreeRuntimeSessionOptions {
            options: self.options,
        }
    }
}

pub struct IreeRuntimeSession {
    pub(crate) session_ptr: *mut iree_runtime_session_t,
}

impl IreeRuntimeSession {
    pub fn create_with_device(
        instance: &IreeRuntimeInstance,
        options: &IreeRuntimeSessionOptions,
        device: &IreeHalDevice,
        allocator: &IreeAllocator,
    ) -> Result<Self, IreeError> {
        let mut session_ptr = std::mem::MaybeUninit::<*mut iree_runtime_session_t>::uninit();

        unsafe {
            let status = iree_runtime_session_create_with_device(
                instance.instance_ptr,
                &options.options,
                device.device_ptr,
                allocator.allocator,
                session_ptr.as_mut_ptr(),
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &instance.host_allocator(),
                ));
            }
        }

        Ok(Self {
            session_ptr: unsafe { session_ptr.assume_init() },
        })
    }

    pub fn device_allocator(&self) -> IreeHalAllocator {
        let allocator_ptr = unsafe { iree_runtime_session_device_allocator(self.session_ptr) };
        IreeHalAllocator { allocator_ptr }
    }

    pub fn get_call_by_name(&self, full_name: &str) -> Result<IreeRuntimeCall, IreeError> {
        let mut call = iree_runtime_call_t::default();
        unsafe {
            let status = iree_runtime_call_initialize_by_name(
                self.session_ptr,
                iree_string_view_t {
                    data: full_name.as_ptr() as *const i8,
                    size: full_name.len(),
                },
                &mut call,
            );

            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }

            Ok(IreeRuntimeCall { call })
        }
    }

    pub fn append_bytecode_module_from_memory(
        &self,
        module_data: &[u8],
        allocator: &IreeAllocator,
    ) -> Result<(), IreeError> {
        let module_data = iree_const_byte_span_t {
            data: module_data.as_ptr() as _,
            data_length: module_data.len() as _,
        };
        unsafe {
            let status = iree_runtime_session_append_bytecode_module_from_memory(
                self.session_ptr,
                module_data,
                allocator.allocator,
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(IreeStatus { status }, allocator));
            }
        }
        Ok(())
    }
}

impl Drop for IreeRuntimeSession {
    fn drop(&mut self) {
        unsafe {
            iree_runtime_session_release(self.session_ptr);
        }
    }
}
