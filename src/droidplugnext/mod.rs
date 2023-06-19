#[cfg(all(target_os = "android", not(feature = "serde")))]
compile_error!("Serde feature must be enabled for Android");

pub mod adapter;
pub mod manager;
pub mod peripheral;

pub(crate) mod jni;
pub(crate) mod objects;

use once_cell::sync::OnceCell;

static GLOBAL_ADAPTER: OnceCell<adapter::Adapter> = OnceCell::new();

pub fn init(vm: *mut jni::JavaVM) -> crate::Result<()> {
    jni::init_jvm(vm).map_err(|e| crate::Error::Other(e.into()))?;
    GLOBAL_ADAPTER.get_or_try_init(|| adapter::Adapter::new())?;
    Ok(())
}

pub fn global_adapter() -> &'static adapter::Adapter {
    GLOBAL_ADAPTER.get().expect(
        "Droidplug has not been initialized. Please initialize it with btleplug::platform::init().",
    )
}
