pub mod IO{
    use std::fmt::write;


#[derive(Debug,Clone)]
pub enum e_IO{
FailedToRegisterDevice(String),
FailedToUnregisterDevice(String),
UnableToGetDBUS(String),
ShutdownNotReady(String),
Custom(String)
}

impl std::fmt::Display for e_IO{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
        e_IO::FailedToRegisterDevice(x) => write!(f,"[IO ERROR: Failed to register a device interface to DBUS]\n{}",x),
        e_IO::FailedToUnregisterDevice(x) => write!(f,"[IO ERROR: Failed to unregister a device interface to DBUS]\n{}",x),
        e_IO::UnableToGetDBUS(x) => write!(f,"[IO ERROR: Failed to capture the DBUS]\n{}",x),
        e_IO::Custom(x) => write!(f, "[IO ERROR: Custom Error]\n{}",x),
        e_IO::ShutdownNotReady(x) => write!(f, "[IO ERROR: Dbus is busy can't Shutdown]\n{}",x),
        }
    }


}

pub type r_IO<T> = Result<T,e_IO>;



}