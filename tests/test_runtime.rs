#[cfg(test)]
mod tests {
    use iree_rs::types::{
        allocator::IreeAllocator,
        runtime::{
            instance::{IreeRuntimeInstance, IreeRuntimeInstanceOptionsBuilder},
            session::{IreeRuntimeSession, IreeRuntimeSessionOptionsBuilder},
        },
    };

    #[test]
    fn test_runtime_instance() {
        let allocator = IreeAllocator::system_allocator();
        let options = IreeRuntimeInstanceOptionsBuilder::default()
            .use_all_available_drivers()
            .build();
        let instance = IreeRuntimeInstance::try_from_options(&options, &allocator).unwrap();
    }

    #[test]
    fn test_runtime_instance_try_create_default_device() {
        let allocator = IreeAllocator::system_allocator();
        let options = IreeRuntimeInstanceOptionsBuilder::default()
            .use_all_available_drivers()
            .build();
        let instance = IreeRuntimeInstance::try_from_options(&options, &allocator).unwrap();
        let device = instance.try_create_default_device("local-task").unwrap();
    }

    #[test]
    fn test_runtime_session() {
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
    }
}
