#[cfg(test)]
mod tests {
    use iree_rs::types::{
        allocator::IreeAllocator,
        bytespan::IreeConstByteSpan,
        hal_buffer::{IreeHalBufferView, IreeHalBufferViewParamsBuilder},
        runtime::{
            instance::{IreeRuntimeInstance, IreeRuntimeInstanceOptionsBuilder},
            session::{IreeRuntimeSession, IreeRuntimeSessionOptionsBuilder},
        },
    };
    use iree_sys::iree::runtime::api::{
        iree_hal_buffer_usage_bits_t_IREE_HAL_BUFFER_USAGE_DEFAULT,
        iree_hal_element_types_t_IREE_HAL_ELEMENT_TYPE_FLOAT_64,
        iree_hal_encoding_types_t_IREE_HAL_ENCODING_TYPE_DENSE_ROW_MAJOR,
        iree_hal_memory_type_bits_t_IREE_HAL_MEMORY_TYPE_DEVICE_LOCAL,
    };

    #[test]
    fn test_hal_buffer_view() {
        let allocator = IreeAllocator::system_allocator();
        let options = IreeRuntimeInstanceOptionsBuilder::default()
            .use_all_available_drivers()
            .build();
        let instance = IreeRuntimeInstance::try_from_options(&options, &allocator).unwrap();
        let device = instance.try_create_default_device("local-task").unwrap();
        let session_options = IreeRuntimeSessionOptionsBuilder::default().build();
        let session = IreeRuntimeSession::create_with_device(
            &instance,
            &session_options,
            &device,
            &allocator,
        )
        .unwrap();

        let data = [1.0, 2.0, 3.0, 4.0];
        let device_allocator = session.device_allocator();
        let byte_span = IreeConstByteSpan::from_slice(&data);

        let buffer_params = IreeHalBufferViewParamsBuilder::default()
            .type_(iree_hal_memory_type_bits_t_IREE_HAL_MEMORY_TYPE_DEVICE_LOCAL.0)
            .usage(iree_hal_buffer_usage_bits_t_IREE_HAL_BUFFER_USAGE_DEFAULT.0)
            .build();

        let buffer = IreeHalBufferView::allocate_buffer(
            &device_allocator,
            &vec![data.len()],
            iree_hal_element_types_t_IREE_HAL_ELEMENT_TYPE_FLOAT_64,
            iree_hal_encoding_types_t_IREE_HAL_ENCODING_TYPE_DENSE_ROW_MAJOR,
            &buffer_params,
            &byte_span,
        )
        .unwrap();
    }
}
