use std::rc::Rc;
use std::collections::{HashMap, BTreeMap};

use dbus;

use common;
use device::Device;
use error::BtError;

static ADAPTER_INTERFACE: &'static str = "org.bluez.Adapter1";

#[derive(Clone, Debug)]
pub struct Adapter {
    conn: Rc<dbus::Connection>,
    object_path: String,
}

#[derive(Clone, Debug)]
pub struct AdapterProperties {
    pub address: String,
    pub name: String,
    pub alias: String,
    pub class: u32,
    pub powered: bool,
    pub discoverable: bool,
    pub discoverable_timeout: u32,
    pub pairable: bool,
    pub pairable_timeout: u32,
    pub discovering: bool,
    pub uuids: Vec<String>,
    pub modalias: Option<String>,
}

impl Adapter {
    pub fn object_path(&self) -> &str {
        &self.object_path
    }

    pub fn get_properties(&self) -> Result<AdapterProperties, BtError> {
        let p = dbus::Props::new(&self.conn, common::SERVICE_NAME, &self.object_path, ADAPTER_INTERFACE, 1000);
        Ok(AdapterProperties::new(try!(p.get_all())))
    }

    pub fn set_alias(&self, val: &str) -> Result<(), BtError> {
        common::dbus_set_property(&self.conn, &self.object_path, ADAPTER_INTERFACE, "Alias", val)
    }

    pub fn set_powered(&self, val: bool) -> Result<(), BtError> {
        common::dbus_set_property(&self.conn, &self.object_path, ADAPTER_INTERFACE, "Powered", val)
    }

    pub fn set_discoverable(&self, val: bool) -> Result<(), BtError> {
        common::dbus_set_property(&self.conn, &self.object_path, ADAPTER_INTERFACE, "Discoverable", val)
    }

    pub fn set_discoverable_timeout(&self, val: u32) -> Result<(), BtError> {
        common::dbus_set_property(&self.conn, &self.object_path, ADAPTER_INTERFACE, "DiscoverableTimeout", val)
    }

    pub fn set_pairable(&self, val: bool) -> Result<(), BtError> {
        common::dbus_set_property(&self.conn, &self.object_path, ADAPTER_INTERFACE, "Pairable", val)
    }

    pub fn set_pairable_timeout(&self, val: u32) -> Result<(), BtError> {
        common::dbus_set_property(&self.conn, &self.object_path, ADAPTER_INTERFACE, "PairableTimeout", val)
    }

    //
    // Methods
    //
    pub fn start_discovery(&self) -> Result<(), BtError> {
        common::dbus_call_method0(&self.conn, &self.object_path, ADAPTER_INTERFACE, "StartDiscovery")
    }

    pub fn stop_discovery(&self) -> Result<(), BtError> {
        common::dbus_call_method0(&self.conn, &self.object_path, ADAPTER_INTERFACE, "StopDiscovery")
    }

    pub fn remove_device(&self, device: &Device) -> Result<(), BtError> {
        // TODO: check for ownership
        //if !device.object_path().starts_with(&self.object_path) {}
        common::dbus_call_method1(&self.conn, &self.object_path, ADAPTER_INTERFACE, "RemoveDevice", device.object_path())
    }
}

impl AdapterProperties {
    fn new(props_map: BTreeMap<String, dbus::MessageItem>) -> AdapterProperties {

        fn _get_prop<'a, T>(props_map: &'a BTreeMap<String, dbus::MessageItem>, name: &str) -> Option<T>
            where T: dbus::FromMessageItem<'a> {
            props_map.get(name).and_then(|x| (x.inner() as Result<T, ()>).ok())
        }

        AdapterProperties {
            address: _get_prop::<&str>(&props_map, "Address").unwrap().to_string(),
            name: _get_prop::<&str>(&props_map, "Name").unwrap().to_string(),
            alias: _get_prop::<&str>(&props_map, "Alias").unwrap().to_string(),
            class: _get_prop(&props_map, "Class").unwrap(),
            powered: _get_prop(&props_map, "Powered").unwrap(),
            discoverable: _get_prop(&props_map, "Discoverable").unwrap(),
            discoverable_timeout: _get_prop(&props_map, "DiscoverableTimeout").unwrap(),
            pairable: _get_prop(&props_map, "Pairable").unwrap(),
            pairable_timeout: _get_prop(&props_map, "PairableTimeout").unwrap(),
            discovering: _get_prop(&props_map, "Discovering").unwrap(),
            uuids: _get_prop::<&[dbus::MessageItem]>(&props_map, "UUIDs").unwrap()
                .iter()
                .map(|x| (x.inner() as Result<&str, ()>).unwrap().to_string())
                .collect(),
            modalias: _get_prop::<&str>(&props_map, "Modalias").map(|x| x.to_string()),
        }
    }
}

pub fn get_adapters(conn: Rc<dbus::Connection>) -> Result<Vec<Adapter>, BtError> {
    let mut adapters: Vec<Adapter> = Vec::new();

    let msg = try!(
        dbus::Message::new_method_call(common::SERVICE_NAME, "/", "org.freedesktop.DBus.ObjectManager", "GetManagedObjects")
            .map_err(BtError::DBusInternal)
    );
    let resp = try!(conn.send_with_reply_and_block(msg, 1000));
    let objects = resp.get_items().pop().unwrap();
    let objects: &[dbus::MessageItem] = objects.inner().unwrap();

    fn _get_prop<'a, T>(props_map: &HashMap<&str, &'a dbus::MessageItem>, name: &str) -> Option<T>
        where T: dbus::FromMessageItem<'a> {
        props_map.get(name).and_then(|x| (x.inner() as Result<T, ()>).ok())
    }

    for obj in objects {
        let (path, interfaces) = obj.inner().unwrap();
        let path: &str = path.inner().unwrap();
        let interfaces: &[dbus::MessageItem] = interfaces.inner().unwrap();

        for interface in interfaces {
            let (name, _) = interface.inner().unwrap();
            let name: &str = name.inner().unwrap();

            if name == ADAPTER_INTERFACE {
                adapters.push(Adapter {
                    conn: conn.clone(),
                    object_path: path.to_string(),
                });
            }
        }
    }

    Ok(adapters)
}

pub fn find_adapter(conn: Rc<dbus::Connection>, name_or_addr: Option<&str>) -> Result<Option<Adapter>, BtError> {
    let adapters = try!(get_adapters(conn));

    if let Some(name_or_addr) = name_or_addr {
        for adapter in adapters {
            let p = try!(adapter.get_properties());
            if p.address == name_or_addr || p.alias == name_or_addr || p.name == name_or_addr {
                return Ok(Some(adapter));
            }
        }
        return Ok(None);
    }

    Ok(adapters.into_iter().next())
}
