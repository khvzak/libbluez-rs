extern crate dbus;

use std::rc::Rc;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Connection {
    dbus: Rc<dbus::Connection>,
}

impl Connection {
    pub fn new() -> Result<Self, error::BtError> {
        Ok(Connection { dbus: Rc::new(try!(dbus::Connection::get_private(dbus::BusType::System))) })
    }
}

impl Deref for Connection {
    type Target = dbus::Connection;

    fn deref(&self) -> &dbus::Connection {
        &self.dbus
    }
}

pub mod adapter;
pub mod device;
pub mod error;

mod common;
