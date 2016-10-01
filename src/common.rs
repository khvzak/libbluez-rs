use dbus;

use error::BtError;

pub static SERVICE_NAME: &'static str = "org.bluez";

pub fn dbus_get_managed_objects<T, F>(conn: &super::Connection,
                                      path: &str,
                                      iface: &str,
                                      f: F) -> Result<Vec<T>, BtError> where F: Fn(super::Connection, &str) -> T {
    let mut path = path.to_string();
    if !path.ends_with("/") { path.push_str("/"); }
    let mut objects_vec = Vec::new();

    let msg = try!(
        dbus::Message::new_method_call(SERVICE_NAME, "/", "org.freedesktop.DBus.ObjectManager", "GetManagedObjects")
            .map_err(BtError::DBusInternal)
    );
    let resp = try!(conn.send_with_reply_and_block(msg, 1000));
    let objects = resp.get_items().pop().unwrap();
    let objects: &[dbus::MessageItem] = objects.inner().unwrap();

    for obj in objects {
        let (obj_path, obj_ifaces) = obj.inner().unwrap();
        let obj_path: &str = obj_path.inner().unwrap();
        let obj_ifaces: &[dbus::MessageItem] = obj_ifaces.inner().unwrap();

        for obj_iface in obj_ifaces {
            let (obj_iface_name, _) = obj_iface.inner().unwrap();
            let obj_iface_name: &str = obj_iface_name.inner().unwrap();

            if obj_iface_name == iface && obj_path.starts_with(&path) {
                objects_vec.push(f(conn.clone(), obj_path));
            }
        }
    }

    Ok(objects_vec)
}

pub fn dbus_get_property<'a, T>(conn: &super::Connection,
                                object_path: &str,
                                interface: &str,
                                prop_name: &str) -> Result<dbus::MessageItem, BtError> {
    let p = dbus::Props::new(conn, SERVICE_NAME, object_path, interface, 1000);
    Ok(try!(p.get(prop_name)))
}

pub fn dbus_set_property<T>(conn: &super::Connection,
                            object_path: &str,
                            interface: &str,
                            prop_name: &str,
                            prop_val: T) -> Result<(), BtError> where T: Into<dbus::MessageItem> {
    let p = dbus::Props::new(conn, SERVICE_NAME, object_path, interface, 1000);
    Ok(try!(p.set(prop_name, prop_val.into())))
}

pub fn dbus_call_method0(conn: &super::Connection,
                         object_path: &str,
                         interface: &str,
                         method_name: &str) -> Result<(), BtError> {
    let m = try!(
        dbus::Message::new_method_call(SERVICE_NAME, object_path, interface, method_name)
            .map_err(BtError::DBusInternal)
    );
    try!(conn.send_with_reply_and_block(m, 60000));
    Ok(())
}

pub fn dbus_call_method1<T>(conn: &super::Connection,
                            object_path: &str,
                            interface: &str,
                            method_name: &str,
                            method_arg1: T) -> Result<(), BtError> where T: dbus::arg::Append {
    let mut m = try!(
        dbus::Message::new_method_call(SERVICE_NAME, object_path, interface, method_name)
            .map_err(BtError::DBusInternal)
    );
    m = m.append1(method_arg1);
    try!(conn.send_with_reply_and_block(m, 60000));
    Ok(())
}

pub fn dbus_call_method2<T1, T2>(conn: &super::Connection,
                                 object_path: &str,
                                 interface: &str,
                                 method_name: &str,
                                 method_arg1: T1,
                                 method_arg2: T2) -> Result<(), BtError>
                                                  where T1: dbus::arg::Append, T2: dbus::arg::Append {
    let mut m = try!(
        dbus::Message::new_method_call(SERVICE_NAME, object_path, interface, method_name)
            .map_err(BtError::DBusInternal)
    );
    m = m.append2(method_arg1, method_arg2);
    try!(conn.send_with_reply_and_block(m, 60000));
    Ok(())
}
