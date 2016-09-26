use dbus;

use error::BtError;

pub static SERVICE_NAME: &'static str = "org.bluez";

pub fn dbus_get_property<'a, T>(conn: &dbus::Connection,
                           object_path: &str,
                           interface: &str,
                           prop_name: &str) -> Result<dbus::MessageItem, BtError> {
    let p = dbus::Props::new(conn, SERVICE_NAME, object_path, interface, 1000);
    Ok(try!(p.get(prop_name)))
}

pub fn dbus_set_property<T>(conn: &dbus::Connection,
                       object_path: &str,
                       interface: &str,
                       prop_name: &str,
                       prop_val: T) -> Result<(), BtError> where T: Into<dbus::MessageItem> {
    let p = dbus::Props::new(conn, SERVICE_NAME, object_path, interface, 1000);
    Ok(try!(p.set(prop_name, prop_val.into())))
}

pub fn dbus_call_method0(conn: &dbus::Connection,
                         object_path: &str,
                         interface: &str,
                         method_name: &str) -> Result<(), BtError> {
    let m = try!(
        dbus::Message::new_method_call(SERVICE_NAME, object_path, interface, method_name)
            .map_err(BtError::DBusInternal)
    );
    try!(conn.send_with_reply_and_block(m, 1000));
    Ok(())
}

pub fn dbus_call_method1<T>(conn: &dbus::Connection,
                         object_path: &str,
                         interface: &str,
                         method_name: &str,
                         method_arg1: T) -> Result<(), BtError> where T: dbus::arg::Append {
    let mut m = try!(
        dbus::Message::new_method_call(SERVICE_NAME, object_path, interface, method_name)
            .map_err(BtError::DBusInternal)
    );
    m = m.append1(method_arg1);
    try!(conn.send_with_reply_and_block(m, 1000));
    Ok(())
}
