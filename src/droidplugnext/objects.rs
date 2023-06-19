use j4rs::{Instance, InvocationArg, JavaClass, Jvm};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    str::FromStr,
};
use uuid::Uuid;

use super::jni::JObjectWrapper;
use crate::{
    api::{AddressType, BDAddr, ParseBDAddrError, PeripheralProperties, ScanFilter},
    Result,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_cr as serde;

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_cr")
)]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ScanResult {
    pub address: String,
    pub addressType: Option<i32>,
    pub localName: Option<String>,
    pub txPowerLevel: Option<i32>,
    pub rssi: Option<i32>,
    pub manufacturerData: HashMap<i32, Vec<u8>>,
    pub serviceData: HashMap<String, Vec<u8>>,
    pub services: Vec<String>,
}

impl Display for ScanResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "address: {:?}, localName: {:?}",
            self.address,
            self.localName.clone().unwrap_or(String::from("unknown")),
        )
    }
}

impl TryFrom<ScanResult> for PeripheralProperties {
    type Error = ParseBDAddrError;

    fn try_from(value: ScanResult) -> core::result::Result<Self, Self::Error> {
        let address = BDAddr::from_str(value.address.as_str())?;
        let address_type = match value.addressType {
            Some(address_type) => address_type.try_into().ok(),
            None => None,
        };
        let tx_power_level = match value.txPowerLevel {
            Some(tx_power_power) => Some(tx_power_power as i16),
            None => None,
        };
        let rssi = match value.rssi {
            Some(rssi) => Some(rssi as i16),
            None => None,
        };
        let manufacturer_data = value
            .manufacturerData
            .into_iter()
            .map(|(key, value)| (key as u16, value))
            .collect();
        let service_data = value
            .serviceData
            .into_iter()
            .filter_map(|(key, value)| Uuid::parse_str(&key).map(|uuid| (uuid, value)).ok())
            .collect();
        let services = value
            .services
            .into_iter()
            .filter_map(|service| Uuid::parse_str(&service).ok())
            .collect();
        Ok(Self {
            address,
            address_type,
            local_name: value.localName,
            tx_power_level,
            rssi,
            manufacturer_data,
            service_data,
            services,
        })
    }
}

impl TryFrom<ScanResult> for (BDAddr, Option<PeripheralProperties>) {
    type Error = ParseBDAddrError;

    fn try_from(value: ScanResult) -> core::result::Result<Self, Self::Error> {
        let address = BDAddr::from_str(value.address.as_str())?;
        let properties = value.try_into().ok();
        Ok((address, properties))
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ParseAddressTypeError {
    #[error("Invalid")]
    Invalid,
}

impl TryFrom<i32> for AddressType {
    type Error = ParseAddressTypeError;
    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(AddressType::Random),
            1 => Ok(AddressType::Public),
            _ => Err(ParseAddressTypeError::Invalid),
        }
    }
}

pub trait JScanFilter {
    fn new(jvm: &Jvm, filter: ScanFilter) -> Result<JObjectWrapper>;
}

impl JScanFilter for JObjectWrapper {
    fn new(jvm: &Jvm, filter: ScanFilter) -> Result<JObjectWrapper> {
        let uuids: Vec<InvocationArg> = filter
            .services
            .into_iter()
            .filter_map(|service| InvocationArg::try_from(service.to_string()).ok())
            .collect();
        let uuids = jvm.create_java_array(JavaClass::String.get_class_str(), &uuids)?;
        let filter = jvm
            .create_instance(
                "com.nonpolynomial.btleplug.droidplug.ScanFilter",
                &[uuids.into()],
            )?
            .java_object();
        Ok(JObjectWrapper { internal: filter })
    }
}

impl TryFrom<JObjectWrapper> for InvocationArg {
    type Error = j4rs::errors::J4RsError;

    fn try_from(value: JObjectWrapper) -> core::result::Result<Self, Self::Error> {
        InvocationArg::try_from(Instance::from_jobject(value.internal))
    }
}
