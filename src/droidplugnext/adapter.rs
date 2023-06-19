use std::{
    fmt::{Debug, Formatter},
    pin::Pin,
    sync::Arc,
};

use async_trait::async_trait;
use futures::Stream;
use j4rs::{self, jni_sys::jobject, prelude::*, InvocationArg, JavaClass};

use super::{
    jni::JObjectWrapper,
    objects::*,
    peripheral::{Peripheral, PeripheralId},
};
use crate::{
    api::{BDAddr, Central, CentralEvent, PeripheralProperties, ScanFilter},
    common::adapter_manager::AdapterManager,
    Error, Result,
};

#[derive(Clone)]
pub struct Adapter {
    internal: jobject,
    manager: Arc<AdapterManager<Peripheral>>,
}

unsafe impl Send for Adapter {}
unsafe impl Sync for Adapter {}

impl Debug for Adapter {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct("Adapter")
            .field("manager", &self.manager)
            .finish()
    }
}

impl Adapter {
    pub(crate) fn new() -> Result<Self> {
        let jvm = Jvm::attach_thread()?;
        let internal = jvm.create_instance("com.nonpolynomial.btleplug.droidplug.Adapter", &[])?;
        let uuids = jvm.create_java_array(JavaClass::String.get_class_str(), &[])?;
        let uuids = InvocationArg::from(uuids);
        jvm.create_instance("com.nonpolynomial.btleplug.droidplug.ScanFilter", &[uuids])?;
        let internal = internal.java_object();
        let manager = Arc::new(AdapterManager::default());
        Ok(Self { internal, manager })
    }

    pub(crate) fn report_scan_result(&self, scan_result: ScanResult) -> Result<Peripheral> {
        let (address, properties): (BDAddr, Option<PeripheralProperties>) =
            scan_result.try_into()?;
        match self.manager.peripheral(&PeripheralId(address)) {
            Some(p) => match properties {
                Some(properties) => {
                    self.report_properties(&p, properties, false);
                    Ok(p)
                }
                None => Err(Error::DeviceNotFound),
            },
            None => match properties {
                Some(properties) => {
                    let p = self.add(address)?;
                    self.report_properties(&p, properties, true);
                    Ok(p)
                }
                None => Err(Error::DeviceNotFound),
            },
        }
    }

    fn get_instance(&self) -> Result<Instance> {
        Instance::from_jobject_with_global_ref(self.internal.clone())
            .map_err(|e| Error::Other(e.into()))
    }

    fn add(&self, address: BDAddr) -> Result<Peripheral> {
        let peripheral: Peripheral = address.into();
        self.manager.add_peripheral(peripheral.clone());
        Ok(peripheral)
    }

    fn report_properties(
        &self,
        peripheral: &Peripheral,
        properties: PeripheralProperties,
        new: bool,
    ) {
        peripheral.report_properties(properties.clone());
        self.manager.emit(if new {
            CentralEvent::DeviceDiscovered(PeripheralId(properties.address))
        } else {
            CentralEvent::DeviceUpdated(PeripheralId(properties.address))
        });
        self.manager
            .emit(CentralEvent::ManufacturerDataAdvertisement {
                id: PeripheralId(properties.address),
                manufacturer_data: properties.manufacturer_data,
            });
        self.manager.emit(CentralEvent::ServiceDataAdvertisement {
            id: PeripheralId(properties.address),
            service_data: properties.service_data,
        });
        self.manager.emit(CentralEvent::ServicesAdvertisement {
            id: PeripheralId(properties.address),
            services: properties.services,
        });
    }
}

#[async_trait]
impl Central for Adapter {
    type Peripheral = Peripheral;

    /// Retrieve a stream of `CentralEvent`s. This stream will receive notifications when events
    /// occur for this Central module. See [`CentralEvent`] for the full set of possible events.
    async fn events(&self) -> Result<Pin<Box<dyn Stream<Item = CentralEvent> + Send>>> {
        Ok(self.manager.event_stream())
    }

    /// Starts a scan for BLE devices. This scan will generally continue until explicitly stopped,
    /// although this may depend on your Bluetooth adapter. Discovered devices will be announced
    /// to subscribers of `events` and will be available via `peripherals()`.
    /// The filter can be used to scan only for specific devices. While some implementations might
    /// ignore (parts of) the filter and make additional devices available, other implementations
    /// might require at least one filter for security reasons. Cross-platform code should provide
    /// a filter, but must be able to handle devices, which do not fit into the filter.
    async fn start_scan(&self, filter: ScanFilter) -> Result<()> {
        let jvm = Jvm::attach_thread()?;
        let filter = JObjectWrapper::new(&jvm, filter)?;
        let filter = filter.try_into()?;
        jvm.invoke(&self.get_instance()?, "startScan", &[filter])
            .map(|_| ())
            .map_err(|e| e.into())
    }

    /// Stops scanning for BLE devices.
    async fn stop_scan(&self) -> Result<()> {
        let jvm = Jvm::attach_thread()?;
        jvm.invoke(&self.get_instance()?, "stopScan", &[])
            .map(|_| ())
            .map_err(|e| e.into())
    }

    /// Returns the list of [`Peripheral`]s that have been discovered so far. Note that this list
    /// may contain peripherals that are no longer available.
    async fn peripherals(&self) -> Result<Vec<Self::Peripheral>> {
        Ok(self.manager.peripherals())
    }

    /// Returns a particular [`Peripheral`] by its address if it has been discovered.
    async fn peripheral(&self, id: &PeripheralId) -> Result<Self::Peripheral> {
        self.manager.peripheral(id).ok_or(Error::DeviceNotFound)
    }

    /// Add a [`Peripheral`] from a MAC address without a scan result. Not supported on all Bluetooth systems.
    async fn add_peripheral(&self, address: &PeripheralId) -> Result<Self::Peripheral> {
        self.add(address.0)
    }

    /// Get information about the Bluetooth adapter being used, such as the model or type.
    ///
    /// The details of this are platform-specific andyou should not attempt to parse it, but it may
    /// be useful for debug logs.
    async fn adapter_info(&self) -> Result<String> {
        Ok("Android".to_string())
    }
}
