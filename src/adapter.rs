use std::collections::BTreeMap;
use std::time::Instant;

use dbus;

use common;
use device::{self, Device};
use error::BtError;

pub static ADAPTER_INTERFACE: &'static str = "org.bluez.Adapter1";

#[derive(Clone, Debug)]
pub struct Adapter {
    conn: super::Connection,
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
    pub fn conn(&self) -> &super::Connection {
        &self.conn
    }

    pub fn object_path(&self) -> &str {
        &self.object_path
    }

    //
    // Properties
    //
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

    pub fn start_discovery_session<F>(&self, duration: u32, mut f: F) -> Result<(), BtError> where F: FnMut(Device) -> () {
        let conn = self.conn();

        let filter1 = format!("sender='{}',interface='org.freedesktop.DBus.ObjectManager',member='InterfacesAdded'", common::SERVICE_NAME);
        let filter2 = format!("sender='{}',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged'", common::SERVICE_NAME);

        try!(conn.add_match(&filter1));
        try!(conn.add_match(&filter2));

        try!(self.start_discovery());

        let now = Instant::now();

        'outer: for i in conn.iter(100) {
            if let dbus::ConnectionItem::Signal(ref s) = i {
                let member = s.member().unwrap();

                if &*member == "PropertiesChanged" && &*s.path().unwrap() == self.object_path() {
                    let items = s.get_items();

                    let iface = items.get(0).unwrap();
                    let iface: &str = iface.inner().unwrap();

                    if iface == ADAPTER_INTERFACE {
                        let props = items.get(1).unwrap();
                        let props: &[dbus::MessageItem] = props.inner().unwrap();

                        for p in props {
                            let (name, val) = p.inner().unwrap();
                            let name: &str = name.inner().unwrap();

                            if name == "Discovering" {
                                let val: bool = (val.inner() as Result<&dbus::MessageItem, ()>).unwrap().inner().unwrap();
                                if !val {
                                    break 'outer;
                                }
                            }
                        }
                    }
                }

                else if &*member == "InterfacesAdded" {
                    let items = s.get_items();

                    let obj_path = items.get(0).unwrap();
                    let obj_path: &str = obj_path.inner().unwrap();

                    let dict = items.get(1).unwrap();
                    let dict: &[dbus::MessageItem] = dict.inner().unwrap();

                    for kv in dict {
                        let (iface, _) = kv.inner().unwrap();
                        let iface: &str = iface.inner().unwrap();

                        if iface == device::DEVICE_INTERFACE {
                            let device = Device::new(conn, obj_path);
                            if device.adapter_object_path() == self.object_path() {
                                f(device);
                            }
                        }
                    }
                }
            }

            if duration > 0 && now.elapsed().as_secs() >= duration as u64 {
                try!(self.stop_discovery());
                break 'outer;
            }
        }

        try!(conn.remove_match(&filter1));
        try!(conn.remove_match(&filter2));

        Ok(())
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

pub fn get_adapters(conn: &super::Connection) -> Result<Vec<Adapter>, BtError> {
    common::dbus_get_managed_objects(conn,
                                     "/",
                                     ADAPTER_INTERFACE,
                                     |conn, obj_path| Adapter { conn: conn.clone(), object_path: obj_path.to_string() }
    )
}

pub fn find_adapter(conn: &super::Connection, name_or_addr: Option<&str>) -> Result<Option<Adapter>, BtError> {
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
