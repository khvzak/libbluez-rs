use std::rc::Rc;
use std::collections::BTreeMap;

use dbus;

use common;
use error::BtError;

static DEVICE_INTERFACE: &'static str = "org.bluez.Device1";

#[derive(Clone, Debug)]
pub struct Device {
    conn: Rc<dbus::Connection>,
    object_path: String,
}

#[derive(Clone, Debug)]
pub struct DeviceProperties {
    pub address: String,
    pub name: Option<String>,
    pub alias: String,
    pub icon: Option<String>,
    pub class: Option<u32>,
    pub appearance: Option<u16>,
    pub uuids: Vec<String>,
    pub paired: bool,
    pub connected: bool,
    pub trusted: bool,
    pub blocked: bool,
    pub legacy_pairing: bool,
    pub modalias: Option<String>,
    pub rssi: Option<i16>,
    // TODO: ManufacturerData, ServiceData, GattServices
}

impl Device {
    pub fn object_path(&self) -> &str {
        &self.object_path
    }

    pub fn get_properties(&self) -> Result<DeviceProperties, BtError> {
        let p = dbus::Props::new(&self.conn, common::SERVICE_NAME, &self.object_path, DEVICE_INTERFACE, 1000);
        Ok(DeviceProperties::new(try!(p.get_all())))
    }

    pub fn set_alias(&self, val: &str) -> Result<(), BtError> {
        common::dbus_set_property(&self.conn, &self.object_path, DEVICE_INTERFACE, "Alias", val)
    }

    pub fn set_trusted(&self, val: bool) -> Result<(), BtError> {
        common::dbus_set_property(&self.conn, &self.object_path, DEVICE_INTERFACE, "Trusted", val)
    }

    pub fn set_blocked(&self, val: bool) -> Result<(), BtError> {
        common::dbus_set_property(&self.conn, &self.object_path, DEVICE_INTERFACE, "Blocked", val)
    }

    //
    // Methods
    //
    pub fn connect(&self) -> Result<(), BtError> {
        common::dbus_call_method0(&self.conn, &self.object_path, DEVICE_INTERFACE, "Connect")
    }

    pub fn disconnect(&self) -> Result<(), BtError> {
        common::dbus_call_method0(&self.conn, &self.object_path, DEVICE_INTERFACE, "Disconnect")
    }

    pub fn connect_profile(&self, uuid: &str) -> Result<(), BtError> {
        common::dbus_call_method1(&self.conn, &self.object_path, DEVICE_INTERFACE, "ConnectProfile", uuid)
    }

    pub fn disconnect_profile(&self, uuid: &str) -> Result<(), BtError> {
        common::dbus_call_method1(&self.conn, &self.object_path, DEVICE_INTERFACE, "DisconnectProfile", uuid)
    }

    pub fn pair(&self) -> Result<(), BtError> {
        common::dbus_call_method0(&self.conn, &self.object_path, DEVICE_INTERFACE, "Pair")
    }

    pub fn cancel_pairing(&self) -> Result<(), BtError> {
        common::dbus_call_method0(&self.conn, &self.object_path, DEVICE_INTERFACE, "CancelPairing")
    }
}

impl DeviceProperties {
    fn new(props_map: BTreeMap<String, dbus::MessageItem>) -> DeviceProperties {

        fn _get_prop<'a, T>(props_map: &'a BTreeMap<String, dbus::MessageItem>, name: &str) -> Option<T>
            where T: dbus::FromMessageItem<'a> {
            props_map.get(name).and_then(|x| (x.inner() as Result<T, ()>).ok())
        }

        DeviceProperties {
            address: _get_prop::<&str>(&props_map, "Address").unwrap().to_string(),
            name: _get_prop::<&str>(&props_map, "Name").map(|x| x.to_string()),
            alias: _get_prop::<&str>(&props_map, "Alias").unwrap().to_string(),
            icon: _get_prop::<&str>(&props_map, "Icon").map(|x| x.to_string()),
            class: _get_prop(&props_map, "Class"),
            appearance: _get_prop(&props_map, "Appearance"),
            uuids: _get_prop::<&[dbus::MessageItem]>(&props_map, "UUIDs").unwrap_or(&Vec::new())
                .iter()
                .map(|x| (x.inner() as Result<&str, ()>).unwrap().to_string())
                .collect(),
            paired: _get_prop(&props_map, "Paired").unwrap(),
            connected: _get_prop(&props_map, "Connected").unwrap(),
            trusted: _get_prop(&props_map, "Trusted").unwrap(),
            blocked: _get_prop(&props_map, "Blocked").unwrap(),
            legacy_pairing: _get_prop(&props_map, "LegacyPairing").unwrap(),
            modalias: _get_prop::<&str>(&props_map, "Modalias").map(|x| x.to_string()),
            rssi: _get_prop(&props_map, "RSSI"),
        }
    }
}
