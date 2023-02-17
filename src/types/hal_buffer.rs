use std::fmt::{Display, Error};

use iree_sys::{
    helper::IREE_CHECK_OK,
    iree::runtime::api::{
        iree_hal_buffer_params_t, iree_hal_buffer_usage_t, iree_hal_buffer_view_allocate_buffer,
        iree_hal_buffer_view_format, iree_hal_buffer_view_release, iree_hal_buffer_view_shape,
        iree_hal_buffer_view_t, iree_hal_dim_t, iree_hal_element_types_t,
        iree_hal_encoding_types_t, iree_hal_memory_access_t, iree_hal_memory_type_t,
    },
};

use crate::err::IreeError;

use super::{
    allocator::IreeAllocator, bytespan::IreeConstByteSpan, hal_allocator::IreeHalAllocator,
    status::IreeStatus,
};

pub type IreeHalBufferShape = Vec<iree_hal_dim_t>;

pub struct IreeHalBufferParams {
    params: iree_hal_buffer_params_t,
}

pub struct IreeHalBufferViewParamsBuilder {
    params: iree_hal_buffer_params_t,
}

impl Default for IreeHalBufferViewParamsBuilder {
    fn default() -> Self {
        let params = iree_hal_buffer_params_t::default();
        Self { params }
    }
}

impl IreeHalBufferViewParamsBuilder {
    pub fn build(&self) -> IreeHalBufferParams {
        IreeHalBufferParams {
            params: self.params,
        }
    }

    pub fn type_(&mut self, type_: iree_hal_memory_type_t) -> &mut Self {
        self.params.type_ |= type_;
        self
    }

    pub fn access(&mut self, access: iree_hal_memory_access_t) -> &mut Self {
        self.params.access |= access;
        self
    }

    pub fn usage(&mut self, usage: iree_hal_buffer_usage_t) -> &mut Self {
        self.params.usage |= usage;
        self
    }
}

pub struct IreeHalBufferView {
    pub(crate) buffer_view_ptr: *mut iree_hal_buffer_view_t,
}

impl IreeHalBufferView {
    pub fn allocate_buffer<T>(
        allocator: &IreeHalAllocator,
        shape: &IreeHalBufferShape,
        element_type: iree_hal_element_types_t,
        encoding_type: iree_hal_encoding_types_t,
        params: &IreeHalBufferParams,
        byte_span: &IreeConstByteSpan<T>,
    ) -> Result<Self, IreeError> {
        let mut buffer_view_ptr = std::mem::MaybeUninit::<*mut iree_hal_buffer_view_t>::uninit();
        unsafe {
            let status = iree_hal_buffer_view_allocate_buffer(
                allocator.allocator_ptr,
                shape.len(),
                shape.as_ptr(),
                element_type.0,
                encoding_type.0,
                params.params,
                byte_span.span,
                buffer_view_ptr.as_mut_ptr(),
            );
            if !IREE_CHECK_OK(status) {
                // FIXME: We don't have the host allocator here, so we can't allocate the error message!
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }
        }
        Ok(Self {
            buffer_view_ptr: unsafe { buffer_view_ptr.assume_init() },
        })
    }
    pub fn try_to_string(&self, max_element_count: usize) -> Result<String, IreeError> {
        let mut buffer = vec![0i8; max_element_count * 24]; // assume 24 bytes per element (maybe overkill)
        let mut out_buffer_length = std::mem::MaybeUninit::<usize>::uninit();
        unsafe {
            let status = iree_hal_buffer_view_format(
                self.buffer_view_ptr,
                max_element_count,
                buffer.len(),
                buffer.as_mut_ptr(),
                out_buffer_length.as_mut_ptr(),
            );

            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }
            let buffer_u8 = buffer.drain(..).map(|b| b as u8).collect::<Vec<u8>>();
            Ok(String::from_utf8_lossy(&buffer_u8[..out_buffer_length.assume_init()]).to_string())
        }
    }
    pub fn shape(&self) -> Result<IreeHalBufferShape, IreeError> {
        let mut out_shape: Vec<iree_hal_dim_t> = vec![0; 32]; // assume max rank is 32 (probably overkill!)
        let mut out_shape_rank = std::mem::MaybeUninit::<usize>::uninit();
        unsafe {
            let status = iree_hal_buffer_view_shape(
                self.buffer_view_ptr,
                out_shape.len(),
                out_shape.as_mut_ptr(),
                out_shape_rank.as_mut_ptr(),
            );
            if !IREE_CHECK_OK(status) {
                return Err(IreeError::from_status(
                    IreeStatus { status },
                    &IreeAllocator::system_allocator(),
                ));
            }
            out_shape.truncate(out_shape_rank.assume_init());
        }
        return Ok(out_shape);
    }
}

impl Drop for IreeHalBufferView {
    fn drop(&mut self) {
        unsafe {
            iree_hal_buffer_view_release(self.buffer_view_ptr);
        }
    }
}

impl Display for IreeHalBufferView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shape = self.shape();
        let n_elements = shape.map(|s| s.iter().fold(1, |a, b| a * b));
        let m = n_elements.and_then(|n| self.try_to_string(n));
        match m {
            Ok(m) => write!(f, "{}", m),
            Err(_e) => return Err(Error {}),
        }
    }
}
