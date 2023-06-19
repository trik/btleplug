use std::{
    collections::BTreeSet,
    fmt::{self, Debug, Display, Formatter},
    future::IntoFuture,
    pin::Pin,
    process::Output,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use futures::{Future, Stream, TryFutureExt};
use j4rs::{errors::J4RsError, prelude::jobject, Instance, InvocationArg, Jvm};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_cr as serde;
use tokio::{spawn, sync::oneshot, task};

use crate::{
    api::{
        self, BDAddr, Characteristic, Descriptor, PeripheralProperties, Service, ValueNotification,
        WriteType,
    },
    Error, Result,
};

use super::jni::{JObjectWrapper, JvmWrapper};

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_cr")
)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PeripheralId(pub(super) BDAddr);
impl Display for PeripheralId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt("", f)
    }
}

#[derive(Clone)]
pub struct Peripheral {
    address: BDAddr,
    internal: Arc<Mutex<JObjectWrapper>>,
    properties: Arc<Mutex<Option<PeripheralProperties>>>,
}

impl Peripheral {
    pub(crate) fn report_properties(&self, properties: PeripheralProperties) {
        let mut guard = self.properties.lock().unwrap();
        *guard = Some(properties);
    }

    fn get_internal(&self) -> jobject {
        let guard = self.internal.lock().unwrap();
        (*guard).internal.clone()
    }

    pub fn get_instance(&self) -> Result<Instance> {
        let internal = self.get_internal();
        Instance::from_jobject_with_global_ref(internal).map_err(|e| Error::Other(e.into()))
    }

    // fn with_jobject<C, T>(&self, f: C) -> Result<T>
    // where
    //     C: for<'a> FnOnce(&'a JvmWrapper, &'a JObjectWrapper) -> Result<T>,
    // {
    //     let jvm = Jvm::attach_thread()?;
    //     let jvm = JvmWrapper { jvm };
    //     let internal = self.get_internal();
    //     let internal = JObjectWrapper { internal };
    //     f(&jvm, &internal)
    // }

    // fn with_instance<C, T>(&self, f: C) -> Result<T>
    // where
    //     C: for<'a> FnOnce(&'a Jvm, &'a Instance) -> Result<T>,
    // {
    //     let jvm = Jvm::attach_thread()?;
    //     let instance = self.get_instance()?;
    //     f(&jvm, &instance)
    // }

    // async fn with_instance_async<'a, T, E, F: Future<Output = std::result::Result<T, E>> + 'a>(
    //     &self,
    //     f: impl FnOnce(&'a Jvm, &'a Instance) -> F,
    // ) -> std::result::Result<T, E> {
    //     let jvm = Jvm::attach_thread().unwrap();
    //     let instance = self.get_instance().unwrap();
    //     f(&jvm, &instance).await
    // }

    // async fn do_connect<'a>(&self) -> impl Future<Output = Result<()>> + '_ {
    //     async {
    //         let jvm = Jvm::attach_thread()?;
    //         let instance = self.get_instance()?;
    //         jvm.invoke_async(&instance, "iddio", &[])
    //             .await
    //             .map(|_| ())
    //             .map_err(|_| Error::DeviceNotFound)
    //     }
    // }

    async fn do_connect(obj: JObjectWrapper) -> Result<()> {
        let jvm = Jvm::attach_thread().unwrap();
        let obj = obj.clone().internal;
        let instance = Instance::from_jobject_with_global_ref(obj).unwrap();
        futures::executor::block_on(jvm.invoke_async(&instance, "connect", &[]))?;
        Ok(())
    }
}

impl TryFrom<BDAddr> for Peripheral {
    type Error = J4RsError;

    fn try_from(address: BDAddr) -> core::result::Result<Self, Self::Error> {
        let jvm = Jvm::attach_thread()?;
        let internal = jvm.create_instance(
            "com.nonpolynomial.btleplug.droidplug.Peripheral",
            &[address.to_string().try_into()?],
        )?;
        let internal = internal.java_object();
        Ok(Self {
            address,
            internal: Arc::new(Mutex::new(JObjectWrapper { internal })),
            properties: Arc::new(Mutex::new(None)),
        })
    }
}

impl Debug for Peripheral {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        // write!(fmt, "{:?}", self.internal.as_obj())
        write!(fmt, "")
    }
}

#[async_trait]
impl api::Peripheral for Peripheral {
    /// Returns the unique identifier of the peripheral.
    fn id(&self) -> PeripheralId {
        PeripheralId(self.address)
    }

    /// Returns the MAC address of the peripheral.
    fn address(&self) -> BDAddr {
        self.address
    }

    /// Returns the set of properties associated with the peripheral. These may be updated over time
    /// as additional advertising reports are received.
    async fn properties(&self) -> Result<Option<PeripheralProperties>> {
        let guard = self.properties.lock().unwrap();
        Ok((*guard).clone())
    }

    /// The set of services we've discovered for this device. This will be empty until
    /// `discover_services` is called.
    fn services(&self) -> BTreeSet<Service> {
        BTreeSet::new()
    }

    /// Returns true iff we are currently connected to the device.
    async fn is_connected(&self) -> Result<bool> {
        let jvm = Jvm::attach_thread()?;
        let instance = self.get_instance()?;
        let res = jvm.invoke(&instance, "isConnected", &[])?;
        jvm.to_rust(res).map_err(|e| e.into())
    }

    /// Creates a connection to the device. If this method returns Ok there has been successful
    /// connection. Note that peripherals allow only one connection at a time. Operations that
    /// attempt to communicate with a device will fail until it is connected.
    async fn connect(&self) -> Result<()> {
        let obj = JObjectWrapper {
            internal: self.get_internal().clone(),
        };
        task::spawn(async { Peripheral::do_connect(obj).await })
            .await
            .map_err(|e| Error::Other(e.into()))?
    }

    /// Terminates a connection to the device.
    async fn disconnect(&self) -> Result<()> {
        Err(Error::DeviceNotFound)
    }

    /// Discovers all services for the device, including their characteristics.
    async fn discover_services(&self) -> Result<()> {
        Err(Error::DeviceNotFound)
    }

    /// Write some data to the characteristic. Returns an error if the write couldn't be sent or (in
    /// the case of a write-with-response) if the device returns an error.
    async fn write(
        &self,
        characteristic: &Characteristic,
        data: &[u8],
        write_type: WriteType,
    ) -> Result<()> {
        Err(Error::DeviceNotFound)
    }

    /// Sends a read request to the device. Returns either an error if the request was not accepted
    /// or the response from the device.
    async fn read(&self, characteristic: &Characteristic) -> Result<Vec<u8>> {
        Err(Error::DeviceNotFound)
    }

    /// Enables either notify or indicate (depending on support) for the specified characteristic.
    async fn subscribe(&self, characteristic: &Characteristic) -> Result<()> {
        Err(Error::DeviceNotFound)
    }

    /// Disables either notify or indicate (depending on support) for the specified characteristic.
    async fn unsubscribe(&self, characteristic: &Characteristic) -> Result<()> {
        Err(Error::DeviceNotFound)
    }

    /// Returns a stream of notifications for characteristic value updates. The stream will receive
    /// a notification when a value notification or indication is received from the device.
    /// The stream will remain valid across connections and can be queried before any connection
    /// is made.
    async fn notifications(&self) -> Result<Pin<Box<dyn Stream<Item = ValueNotification> + Send>>> {
        Err(Error::DeviceNotFound)
    }

    /// Write some data to the descriptor. Returns an error if the write couldn't be sent or (in
    /// the case of a write-with-response) if the device returns an error.
    async fn write_descriptor(&self, descriptor: &Descriptor, data: &[u8]) -> Result<()> {
        Err(Error::DeviceNotFound)
    }

    /// Sends a read descriptor request to the device. Returns either an error if the request
    /// was not accepted or the response from the device.
    async fn read_descriptor(&self, descriptor: &Descriptor) -> Result<Vec<u8>> {
        Err(Error::DeviceNotFound)
    }
}
