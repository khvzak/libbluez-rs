use std::fmt;
use std::rc::Rc;

use dbus;

use common;
use device::Device;
use error::BtError;

pub static AGENT_INTERFACE: &'static str = "org.bluez.Agent1";
pub static AGENT_MANAGER_INTERFACE: &'static str = "org.bluez.AgentManager1";
pub static AGENT_MANAGER_OBJ_PATH: &'static str = "/org/bluez";

pub enum AgentCapability {
    DisplayOnly,
    DisplayYesNo,
    KeyboardOnly,
    NoInputNoOutput,
    KeyboardDisplay,
}

impl AgentCapability {
    fn to_str(&self) -> &'static str {
        match *self {
            AgentCapability::DisplayOnly => "DisplayOnly",
            AgentCapability::DisplayYesNo => "DisplayYesNo",
            AgentCapability::KeyboardDisplay => "KeyboardDisplay",
            AgentCapability::KeyboardOnly => "KeyboardOnly",
            AgentCapability::NoInputNoOutput => "NoInputNoOutput",
        }
    }
}

pub enum AgentError {
    Rejected,
    Canceled,
}

impl AgentError {
    fn bluez_error(&self) -> &'static str {
        match *self {
            AgentError::Rejected => "org.bluez.Error.Rejected",
            AgentError::Canceled => "org.bluez.Error.Canceled",
        }
    }
}

pub trait Agent {
    fn get_object_path(&self) -> &str {
        "/io/bluezrs/agent1"
    }

    fn get_capability(&self) -> AgentCapability {
        AgentCapability::KeyboardDisplay
    }

    fn request_pincode(&self, device: Device) -> Result<String, AgentError>;
    fn display_pincode(&self, device: Device, pincode: &str) -> Result<(), AgentError>;
    fn request_passkey(&self, device: Device) -> Result<u32, AgentError>;
    fn display_passkey(&self, device: Device, passkey: u32, entered: u16);
    fn request_confirmation(&self, device: Device, passkey: u32) -> Result<(), AgentError>;
    fn request_authorization(&self, device: Device) -> Result<(), AgentError>;
    fn authorize_service(&self, device: Device, uuid: &str) -> Result<(), AgentError>;
    fn cancel(&self);
    fn release(&self);
}

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Agent(object_path: \"{}\", capability: \"{}\")", self.get_object_path(), self.get_capability().to_str())
    }
}

type SharedAgentT = Rc<Box<Agent>>;

#[derive(Copy, Clone, Default, Debug)]
struct TData;
impl dbus::tree::DataType for TData {
    type ObjectPath = SharedAgentT;
    type Property = ();
    type Interface = ();
    type Method = Option<super::Connection>;
    type Signal = ();
}

pub struct AgentManager {
    conn: super::Connection,
    tree: dbus::tree::Tree<dbus::tree::MTFn<TData>, TData>,
    agent: SharedAgentT,
}

impl AgentManager {
    pub fn new(conn: &super::Connection, agent: Box<Agent>) -> AgentManager {
        let agent = Rc::new(agent);

        let f = dbus::tree::Factory::new_fn();

        let tree = f.tree().add(
            f.object_path(agent.get_object_path().to_string(), agent.clone()).introspectable().add(
                f.interface(AGENT_INTERFACE, ())
                    .add_m(
                        f.method("RequestPinCode", Some(conn.clone()), move |m| {
                            let conn = (m.method.get_data() as &Option<super::Connection>).as_ref().unwrap();
                            let agent: &SharedAgentT = m.path.get_data();

                            let msg = m.msg;
                            let device_obj_path: dbus::Path = msg.get1().unwrap();
                            let pincode = agent.request_pincode(Device::new(conn, &device_obj_path));

                            match pincode {
                                Ok(pincode) => Ok(vec![m.msg.method_return().append1(pincode)]),
                                Err(e) => Err(dbus::tree::MethodErr::failed(&e.bluez_error()))
                            }
                        }).in_arg(("device", "o")).out_arg("s")
                    )
                    .add_m(
                        f.method("DisplayPinCode", Some(conn.clone()), move |m| {
                            let conn = (m.method.get_data() as &Option<super::Connection>).as_ref().unwrap();
                            let agent: &SharedAgentT = m.path.get_data();

                            let msg = m.msg;
                            let (device_obj_path, pincode): (Option<dbus::Path>, Option<&str>) = msg.get2();
                            let r = agent.display_pincode(Device::new(conn, &device_obj_path.unwrap()), pincode.unwrap());

                            match r {
                                Ok(_) => Ok(vec![m.msg.method_return()]),
                                Err(e) => Err(dbus::tree::MethodErr::failed(&e.bluez_error()))
                            }
                        }).in_arg(("device", "o")).in_arg(("pincode", "s"))
                    )
                    .add_m(
                        f.method("RequestPasskey", Some(conn.clone()), move |m| {
                            let conn = (m.method.get_data() as &Option<super::Connection>).as_ref().unwrap();
                            let agent: &SharedAgentT = m.path.get_data();

                            let msg = m.msg;
                            let device_obj_path: dbus::Path = msg.get1().unwrap();
                            let passkey = agent.request_passkey(Device::new(conn, &device_obj_path));

                            match passkey {
                                Ok(passkey) => Ok(vec![m.msg.method_return().append1(passkey)]),
                                Err(e) => Err(dbus::tree::MethodErr::failed(&e.bluez_error()))
                            }
                        }).in_arg(("device", "o")).out_arg("u")
                    )
                    .add_m(
                        f.method("DisplayPasskey", Some(conn.clone()), move |m| {
                            let conn = (m.method.get_data() as &Option<super::Connection>).as_ref().unwrap();
                            let agent: &SharedAgentT = m.path.get_data();

                            let msg = m.msg;
                            let (device_obj_path, passkey, entered): (Option<dbus::Path>, Option<u32>, Option<u16>) = msg.get3();
                            agent.display_passkey(Device::new(conn, &device_obj_path.unwrap()), passkey.unwrap(), entered.unwrap());

                            Ok(vec![m.msg.method_return()])
                        }).in_arg(("device", "o")).in_arg(("passkey", "u")).in_arg(("entered", "q"))
                    )
                    .add_m(
                        f.method("RequestConfirmation", Some(conn.clone()), move |m| {
                            let conn = (m.method.get_data() as &Option<super::Connection>).as_ref().unwrap();
                            let agent: &SharedAgentT = m.path.get_data();

                            let msg = m.msg;
                            let (device_obj_path, passkey): (Option<dbus::Path>, Option<u32>) = msg.get2();
                            let r = agent.request_confirmation(Device::new(conn, &device_obj_path.unwrap()), passkey.unwrap());

                            match r {
                                Ok(_) => Ok(vec![m.msg.method_return()]),
                                Err(e) => Err(dbus::tree::MethodErr::failed(&e.bluez_error()))
                            }
                        }).in_arg(("device", "o")).in_arg(("passkey", "u"))
                    )
                    .add_m(
                        f.method("RequestAuthorization", Some(conn.clone()), move |m| {
                            let conn = (m.method.get_data() as &Option<super::Connection>).as_ref().unwrap();
                            let agent: &SharedAgentT = m.path.get_data();

                            let msg = m.msg;
                            let device_obj_path: dbus::Path = msg.get1().unwrap();
                            let r = agent.request_authorization(Device::new(conn, &device_obj_path));

                            match r {
                                Ok(_) => Ok(vec![m.msg.method_return()]),
                                Err(e) => Err(dbus::tree::MethodErr::failed(&e.bluez_error()))
                            }
                        }).in_arg(("device", "o"))
                    )
                    .add_m(
                        f.method("AuthorizeService", Some(conn.clone()), move |m| {
                            let conn = (m.method.get_data() as &Option<super::Connection>).as_ref().unwrap();
                            let agent: &SharedAgentT = m.path.get_data();

                            let msg = m.msg;
                            let (device_obj_path, uuid): (Option<dbus::Path>, Option<&str>) = msg.get2();
                            let r = agent.authorize_service(Device::new(conn, &device_obj_path.unwrap()), uuid.unwrap());

                            match r {
                                Ok(_) => Ok(vec![m.msg.method_return()]),
                                Err(e) => Err(dbus::tree::MethodErr::failed(&e.bluez_error()))
                            }
                        }).in_arg(("device", "o")).in_arg(("uuid", "s"))
                    )
                    .add_m(
                        f.method("Cancel", None, move |m| {
                            let agent: &SharedAgentT = m.path.get_data();
                            agent.cancel();
                            Ok(vec![m.msg.method_return()])
                        })
                    )
                    .add_m(
                        f.method("Release", None, move |m| {
                            let agent: &SharedAgentT = m.path.get_data();
                            agent.release();
                            Ok(vec![m.msg.method_return()])
                        })
                    )
        ));

        AgentManager { conn: conn.clone(), tree: tree, agent: agent }
    }

    pub fn register_agent(&self) -> Result<(), BtError> {
        let agent_capabitily = self.agent.get_capability().to_str();
        let agent_obj_path = dbus::Path::new(self.agent.get_object_path()).unwrap();

        try!(self.tree.set_registered(&self.conn, true));
        try!(common::dbus_call_method2(&self.conn, AGENT_MANAGER_OBJ_PATH, AGENT_MANAGER_INTERFACE, "RegisterAgent", agent_obj_path, agent_capabitily));

        Ok(())
    }

    pub fn serve(&self, cb: Option<&Fn() -> bool>) {
        for _ in self.tree.run(&self.conn, self.conn.iter(100)) {
            if let Some(cb) = cb {
                if !cb() { break; }
            }
        }
    }

    pub fn unregister_agent(&self) -> Result<(), BtError> {
        let agent_obj_path = dbus::Path::new(self.agent.get_object_path()).unwrap();
        try!(common::dbus_call_method1(&self.conn, AGENT_MANAGER_OBJ_PATH, AGENT_MANAGER_INTERFACE, "UnregisterAgent", agent_obj_path));
        Ok(())
    }

    pub fn request_default_agent(&self) -> Result<(), BtError> {
        let agent_obj_path = dbus::Path::new(self.agent.get_object_path()).unwrap();
        try!(common::dbus_call_method1(&self.conn, AGENT_MANAGER_OBJ_PATH, AGENT_MANAGER_INTERFACE, "RequestDefaultAgent", agent_obj_path.clone()));
        Ok(())
    }
}
