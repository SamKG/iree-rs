use iree_sys::{helper::*, iree::runtime::api::*};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{ffi::CString, ptr::null_mut, time::Instant};
use tch;

#[derive(Deserialize, Serialize)]
struct Image {
    data: Vec<f32>,
    shape: Vec<i64>,
}

fn load_image() -> Result<tch::Tensor, tch::TchError> {
    let j = serde_json::from_str::<Image>(include_str!("test_image.json")).unwrap();
    Ok(tch::Tensor::of_slice(&j.data).reshape(j.shape.as_slice()))
}

static mut FLATBUFFER_DATA: Lazy<Vec<u8>> = Lazy::new(|| include_bytes!("resnet18.vmfb").to_vec());

unsafe fn iree_runtime_demo_run_session(instance: *mut iree_runtime_instance_t) {
    // TODO(#5724): move device selection into the compiled modules.
    let mut device: *mut iree_hal_device_t = null_mut();

    let s_str = CString::new("local-task").unwrap();
    let string_view = iree_string_view_t {
        data: s_str.as_ptr() as *const i8,
        size: s_str.as_bytes().len(),
    };

    let status =
        iree_runtime_instance_try_create_default_device(instance, string_view, &mut device as _);
    assert!(
        IREE_CHECK_OK(status),
        "status: {}",
        IREE_STATUS_TO_STRING(status)
    );

    let allocator = iree_runtime_instance_host_allocator(instance);

    // Create one session per loaded module to hold the module state.
    let mut session_options = iree_runtime_session_options_t::default();

    iree_runtime_session_options_initialize(&mut session_options as _);

    let mut session: *mut iree_runtime_session_t = null_mut();
    let status = iree_runtime_session_create_with_device(
        instance,
        &session_options as _,
        device,
        allocator,
        &mut session as _,
    );

    assert!(
        IREE_CHECK_OK(status),
        "status: {}",
        IREE_STATUS_TO_STRING(status)
    );
    iree_hal_device_release(device);

    // Load your user module into the session (from memory, from file, etc).

    FLATBUFFER_DATA.push(0);
    let status = iree_runtime_session_append_bytecode_module_from_memory(
        session,
        iree_const_byte_span_t {
            data: FLATBUFFER_DATA.as_ptr() as _,
            data_length: FLATBUFFER_DATA.len(),
        },
        iree_runtime_session_host_allocator(session),
    );

    // let fpath = CString::new("examples/resnet18.vmfb").unwrap();
    // let status = iree_runtime_session_append_bytecode_module_from_file(
    //     session,
    //     fpath.as_ptr() as *const c_char,
    // );

    assert!(
        IREE_CHECK_OK(status),
        "status: {}",
        IREE_STATUS_TO_STRING(status)
    );

    // Run your functions; you should reuse the session to make multiple calls.
    iree_runtime_demo_perform_mul(session);

    iree_runtime_session_release(session);
}

//===----------------------------------------------------------------------===//
// 3. Call a function within a module with buffer views
//===----------------------------------------------------------------------===//

// func.func @simple_mul(%arg0: tensor<4xf32>, %arg1: tensor<4xf32>) ->
// tensor<4xf32>
unsafe fn iree_runtime_demo_perform_mul(session: *mut iree_runtime_session_t) {
    let mut call = iree_runtime_call_t::default();
    let mut str_n = "module.forward";
    let status = iree_runtime_call_initialize_by_name(
        session,
        iree_string_view_t {
            data: str_n.as_ptr() as *const i8,
            size: str_n.len(),
        },
        &mut call as _,
    );

    assert!(
        IREE_CHECK_OK(status),
        "status: {}",
        IREE_STATUS_TO_STRING(status)
    );

    // %arg0: tensor<4xf32>
    let mut arg0: *mut iree_hal_buffer_view_t = null_mut();
    let img = load_image().unwrap();

    let arg0_shape: Vec<iree_hal_dim_t> = img.size().iter().map(|x| *x as _).collect();
    let arg0_data = img.to_kind(tch::Kind::Float);

    let allocator = iree_runtime_session_device_allocator(session);

    let byte_span = iree_const_byte_span_t {
        data: arg0_data.data_ptr() as _,
        data_length: arg0_data.flatten(0, -1).size()[0] as usize * std::mem::size_of::<f32>(),
    };
    println!("byte_span: {:?}", byte_span);

    let mut buff_params = iree_hal_buffer_params_t::default();
    buff_params.type_ = iree_hal_memory_type_bits_t_IREE_HAL_MEMORY_TYPE_DEVICE_LOCAL.0;
    buff_params.access = 0; // fixme: incorrect type?
    buff_params.usage = iree_hal_buffer_usage_bits_t_IREE_HAL_BUFFER_USAGE_DEFAULT.0;

    let status = iree_hal_buffer_view_allocate_buffer(
        allocator,
        arg0_shape.len(),
        arg0_shape.as_ptr() as _,
        iree_hal_element_types_t_IREE_HAL_ELEMENT_TYPE_FLOAT_32.0,
        iree_hal_encoding_types_t_IREE_HAL_ENCODING_TYPE_DENSE_ROW_MAJOR.0,
        buff_params,
        byte_span,
        &mut arg0 as _,
    );

    assert!(
        IREE_CHECK_OK(status),
        "status: {}",
        IREE_STATUS_TO_STRING(status)
    );

    iree_hal_buffer_view_fprint(
        stdout,
        arg0,
        /*max_element_count=*/ 10,
        iree_runtime_session_host_allocator(session),
    );
    iree_runtime_call_inputs_push_back_buffer_view(&mut call as _, arg0);
    iree_hal_buffer_view_release(arg0);

    let start_t = Instant::now();
    let status = iree_runtime_call_invoke(&mut call as _, /*flags=*/ 0);
    let end_t = Instant::now();

    println!("invoke time: {:?}", end_t - start_t);
    assert!(
        IREE_CHECK_OK(status),
        "status: {}",
        IREE_STATUS_TO_STRING(status)
    );

    // -> tensor<4xf32>
    let mut ret0: *mut iree_hal_buffer_view_t = null_mut();
    let status = iree_runtime_call_outputs_pop_front_buffer_view(&mut call as _, &mut ret0 as _);
    assert!(
        IREE_CHECK_OK(status),
        "status: {}",
        IREE_STATUS_TO_STRING(status)
    );

    iree_hal_buffer_view_fprint(
        stdout,
        ret0,
        /*max_element_count=*/ 10,
        iree_runtime_session_host_allocator(session),
    );
    iree_hal_buffer_view_release(ret0);

    iree_runtime_call_deinitialize(&mut call as _);
}

#[cfg(test)]
pub mod test {

    use iree_sys::iree::runtime::api::*;
    use std::ptr::null_mut;

    use crate::*;

    #[test]
    fn main() {
        unsafe {
            let mut instance_options = iree_runtime_instance_options_t::default();
            iree_runtime_instance_options_initialize(&mut instance_options as *mut _);
            iree_runtime_instance_options_use_all_available_drivers(
                &mut instance_options as *mut _,
            );
            let mut instance: *mut iree_runtime_instance_t = null_mut();

            let allocator = iree_allocator_t {
                self_: null_mut(),
                ctl: Some(iree_allocator_system_ctl as _),
            };

            iree_runtime_instance_create(&instance_options, allocator, &mut instance as _);

            // All sessions should share the same instance.
            iree_runtime_demo_run_session(instance);

            iree_runtime_instance_release(instance);
        }
    }
}

fn main() {}
