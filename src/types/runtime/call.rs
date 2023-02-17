use iree_sys::{
    helper::IREE_CHECK_OK,
    iree::runtime::api::{
        iree_hal_buffer_view_t, iree_runtime_call_deinitialize, iree_runtime_call_flags_t,
        iree_runtime_call_initialize_by_name, iree_runtime_call_inputs_push_back_buffer_view,
        iree_runtime_call_invoke, iree_runtime_call_outputs_pop_front_buffer_view,
        iree_runtime_call_t, iree_string_view_t,
    },
};

use crate::{
    err::IreeError,
    types::{allocator::IreeAllocator, hal_buffer::IreeHalBufferView, status::IreeStatus},
};

use super::session::IreeRuntimeSession;

pub struct IreeRuntimeCall {
    pub(crate) call: iree_runtime_call_t,
}
impl IreeRuntimeCall {
    pub fn initialize_by_name(
        session: &IreeRuntimeSession,
        full_name: &String,
    ) -> Result<Self, IreeError> {
        let mut call = iree_runtime_call_t::default();

        unsafe {
            let status = iree_runtime_call_initialize_by_name(
                session.session_ptr,
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
        }

        Ok(Self { call })
    }

    pub fn inputs_push_back_buffer_view(
        &mut self,
        buffer_view: &IreeHalBufferView,
    ) -> Result<(), IreeError> {
        unsafe {
            let status = iree_runtime_call_inputs_push_back_buffer_view(
                &mut self.call,
                buffer_view.buffer_view_ptr,
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }
            Ok(())
        }
    }

    pub fn outputs_pop_front_buffer_view(&mut self) -> Result<IreeHalBufferView, IreeError> {
        let mut ret = std::mem::MaybeUninit::<*mut iree_hal_buffer_view_t>::uninit();
        unsafe {
            let status =
                iree_runtime_call_outputs_pop_front_buffer_view(&mut self.call, ret.as_mut_ptr());

            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }

            Ok(IreeHalBufferView {
                buffer_view_ptr: ret.assume_init(),
            })
        }
    }

    pub fn invoke(&mut self, flags: iree_runtime_call_flags_t) -> Result<(), IreeError> {
        unsafe {
            let status = iree_runtime_call_invoke(&mut self.call, flags);
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }
        }
        Ok(())
    }
}

impl Drop for IreeRuntimeCall {
    fn drop(&mut self) {
        unsafe {
            iree_runtime_call_deinitialize(&mut self.call);
        }
    }
}
