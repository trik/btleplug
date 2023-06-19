pub use j4rs::jni_sys::JavaVM;
use j4rs::{errors::J4RsError, prelude::*, set_java_vm};
use once_cell::sync::OnceCell;

use super::{objects::ScanResult, GLOBAL_ADAPTER};

#[derive(Debug, thiserror::Error)]
pub enum AndroidInitError {
    #[error("JVM init error")]
    JVMInit,
    #[error("JVM class loader init error")]
    JVMClassLoaderInit,
    #[error(transparent)]
    JavaError(#[from] J4RsError),
}

impl From<J4RsError> for crate::Error {
    fn from(err: J4RsError) -> Self {
        Self::Other(Box::new(err))
    }
}

struct JvmWrapper {
    _jvm: Jvm,
}

// Implementing Send and Sync is actually safe and proposed for Android to avoid classloading issues
// when creating new threads.
// https://developer.android.com/training/articles/perf-jni
unsafe impl Sync for JvmWrapper {}
unsafe impl Send for JvmWrapper {}

pub struct JObjectWrapper {
    pub internal: jobject,
}

// Implementing Send and Sync is actually safe and proposed for Android to avoid classloading issues
// when creating new threads.
// https://developer.android.com/training/articles/perf-jni
unsafe impl Sync for JObjectWrapper {}
unsafe impl Send for JObjectWrapper {}

// We need to store both Jvm and class loader to avoid deletion
// of global references
static JVM: OnceCell<JvmWrapper> = OnceCell::new();
static CLASS_LOADER: OnceCell<JObjectWrapper> = OnceCell::new();

pub fn init_jvm(vm: *mut JavaVM) -> Result<(), AndroidInitError> {
    set_java_vm(vm);
    let jvm = Jvm::attach_thread().map_err(|_| AndroidInitError::JVMInit)?;
    let _ = JVM.set(JvmWrapper { _jvm: jvm });
    let jvm = Jvm::attach_thread().map_err(|_| AndroidInitError::JVMInit)?;
    let thread = jvm.invoke_static("java.lang.Thread", "currentThread", &[])?;
    let class_loader = jvm.invoke(&thread, "getContextClassLoader", &[])?;
    CLASS_LOADER
        .set(JObjectWrapper {
            internal: class_loader.java_object(),
        })
        .map_err(|_| AndroidInitError::JVMClassLoaderInit)?;
    Ok(())
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn Java_com_nonpolynomial_btleplug_droidplug_Adapter_reportScanResult(
    env: *mut JNIEnv,
    _class: *const c_void,
    result: jobject,
) {
    let jvm = Jvm::try_from(env).unwrap();
    let result = Instance::from_jobject_with_global_ref(result).unwrap();
    let scan_result: ScanResult = jvm.to_rust(result).unwrap();
    let adapter = GLOBAL_ADAPTER.get().unwrap();
    adapter.report_scan_result(scan_result).unwrap();
}

#[macro_export]
macro_rules! droidplug_init {
    () => {
        droidplug_init!(0x00010006);
    };
    ($version: tt) => {
        #[no_mangle]
        pub extern "C" fn JNI_OnLoad(
            vm: *mut j4rs::jni_sys::JavaVM,
            _: j4rs::jni_sys::jobject,
        ) -> j4rs::jni_sys::jint {
            btleplug::platform::init(vm).unwrap();
            $version.into()
        }
    };
}
