use crate::iree::runtime::api::*;
use std::{ffi::CString, ptr::null_mut};

pub unsafe fn IREE_CHECK_OK(status: *mut iree_status_handle_t) -> bool {
    return status == iree_status_code_e_IREE_STATUS_OK.0 as _;
}

pub unsafe fn IREE_STATUS_TO_STRING(status: *mut iree_status_handle_t) -> String {
    let host_allocator = iree_allocator_t {
        self_: null_mut(),
        ctl: Some(iree_allocator_system_ctl as _),
    };
    let mut out_buffer: *mut i8 = null_mut();
    let mut out_buffer_length: usize = 0;

    iree_status_to_string(
        status,
        &host_allocator as _,
        &mut out_buffer as _,
        &mut out_buffer_length as _,
    );
    return CString::from_raw(out_buffer).to_str().unwrap().to_string();
}
