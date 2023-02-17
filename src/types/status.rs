use iree_sys::{
    helper::IREE_CHECK_OK,
    iree::runtime::api::{iree_status_t, iree_status_to_string},
};

use crate::err::IreeError;

use super::allocator::IreeAllocator;

#[derive(Clone, Copy, Debug)]
pub struct IreeStatus {
    pub(crate) status: iree_status_t,
}

impl From<iree_status_t> for IreeStatus {
    fn from(status: iree_status_t) -> Self {
        Self { status }
    }
}

impl IreeStatus {
    pub fn is_ok(&self) -> bool {
        unsafe { IREE_CHECK_OK(self.status) }
    }
    pub fn to_string(&self, allocator: &IreeAllocator) -> Result<String, IreeError> {
        let mut out_buffer = std::mem::MaybeUninit::<*mut u8>::uninit();
        let mut out_buffer_length = std::mem::MaybeUninit::<usize>::uninit();
        unsafe {
            let tostr_success = iree_status_to_string(
                self.status,
                &allocator.allocator,
                out_buffer.as_mut_ptr() as _,
                out_buffer_length.as_mut_ptr(),
            );
            if !tostr_success {
                return Err("Failed to convert status to string".to_string().into());
            }

            let out_buffer = out_buffer.assume_init();
            let out_buffer_length = out_buffer_length.assume_init();
            let buffer = std::slice::from_raw_parts(out_buffer, out_buffer_length);

            Ok(String::from_utf8(buffer.to_vec())?)
        }
    }
}
