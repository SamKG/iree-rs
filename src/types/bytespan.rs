use iree_sys::iree::runtime::api::iree_const_byte_span_t;

pub struct IreeConstByteSpan<'a, T> {
    pub(crate) span: iree_const_byte_span_t,
    pub(crate) _data: &'a [T], // keep the data alive
}

impl<'a, T> IreeConstByteSpan<'a, T> {
    pub fn from_slice(data: &'a [T]) -> Self {
        Self {
            span: iree_const_byte_span_t {
                data: data.as_ptr() as *const _,
                data_length: data.len() * std::mem::size_of::<T>(),
            },
            _data: data,
        }
    }
}
