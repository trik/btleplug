use std::{
    collections::BTreeSet,
    fmt::{self, Debug, Display, Formatter},
    pin::Pin,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use futures::Stream;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_cr as serde;

use crate::{
    api::{
        self, BDAddr, Characteristic, Descriptor, PeripheralProperties, Service, ValueNotification,
        WriteType,
    },
    Error, Result,
};

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
    properties: Arc<Mutex<Option<PeripheralProperties>>>,
}

impl Peripheral {
    pub(crate) fn report_properties(&self, properties: PeripheralProperties) {
        let mut guard = self.properties.lock().unwrap();
        *guard = Some(properties);
    }
}

impl From<BDAddr> for Peripheral {
    fn from(address: BDAddr) -> Self {
        Self {
            address,
            properties: Arc::new(Mutex::new(None)),
        }
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
        Err(Error::DeviceNotFound)
    }

    /// Creates a connection to the device. If this method returns Ok there has been successful
    /// connection. Note that peripherals allow only one connection at a time. Operations that
    /// attempt to communicate with a device will fail until it is connected.
    async fn connect(&self) -> Result<()> {
        Err(Error::DeviceNotFound)
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
