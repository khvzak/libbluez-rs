use std::{error, fmt};

use dbus;

#[derive(Debug)]
pub enum BtError {
    DBus(dbus::Error),
    DBusInternal(String),
}

impl From<dbus::Error> for BtError {
    fn from(err: dbus::Error) -> BtError {
        BtError::DBus(err)
    }
}

impl fmt::Display for BtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BtError::DBus(ref err) => err.fmt(f),
            BtError::DBusInternal(ref err_msg) => write!(f, "{}", err_msg),
        }
    }
}

impl error::Error for BtError {
    fn description(&self) -> &str {
        match *self {
            BtError::DBus(ref err) => err.description(),
            BtError::DBusInternal(ref err_msg) => err_msg,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            BtError::DBus(ref err) => Some(err),
            BtError::DBusInternal(..) => None,
        }
    }
}
