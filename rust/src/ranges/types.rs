#![allow(dead_code)]

use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum TypeRange {Port, IP, Socket}

pub const MAX_IP: u64 = u64::pow(2, 32) - 1;
pub const MAX_PORT: u64 = u64::pow(2, 16) - 1;
pub const MAX_SOCKET: u64 = u64::pow(2, 32 + 16) - 1;


pub trait Range {

    fn min(&self) -> u64 { 0 }

    fn max(&self) -> u64;

    fn range_of(&self) -> TypeRange;

    fn begin(&self) -> u64;

    fn end(&self) -> u64;

    fn set_end(&mut self, val: u64);

    fn clone_dyn(&self) -> Box<dyn Range>;

}


impl Clone for Box<dyn Range> {

    fn clone(&self) -> Self { self.clone_dyn() }

}

impl fmt::Display for Box<dyn Range> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!("({}, {})", self.begin(), self.end())
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Port { pub begin: u64, pub end: u64}


#[derive(Clone, Copy, PartialEq)]
pub struct IP { pub begin: u64, pub end: u64}


#[derive(Clone, Copy, PartialEq)]
pub struct Socket { pub begin: u64, pub end: u64}


impl Port {

    pub fn new(begin: u64, end: u64) -> Self {  Self { begin, end } }

}


impl IP {

    pub fn new(begin: u64, end: u64) -> Self {  Self { begin, end } }

}


impl Socket {

    pub fn new(begin: u64, end: u64) -> Self {  Self { begin, end } }

}


impl fmt::Debug for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("({}, {})", &self.begin, &self.end))
    }
}


impl fmt::Debug for IP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("({}, {})", &self.begin, &self.end))
    }
}


impl fmt::Debug for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("({}, {})", &self.begin, &self.end))
    }
}


impl Range for Port {

    fn max(&self) -> u64 { MAX_PORT }

    fn range_of(&self) -> TypeRange { TypeRange::Port }

    fn begin(&self) -> u64 { self.begin }

    fn end(&self) -> u64 { self.end  }

    fn set_end(&mut self, val: u64) { self.end = val; }

    fn clone_dyn(&self) -> Box<dyn Range> { Box::new(self.clone()) }

}


impl Range for IP {

    fn max(&self) -> u64 { MAX_IP  }

    fn range_of(&self) -> TypeRange { TypeRange::IP }

    fn begin(&self) -> u64 { self.begin }

    fn end(&self) -> u64 { self.end  }

    fn set_end(&mut self, val: u64) { self.end = val; }

    fn clone_dyn(&self) -> Box<dyn Range> { Box::new(self.clone()) }

}


impl Range for Socket {

    fn max(&self) -> u64 { MAX_SOCKET }

    fn range_of(&self) -> TypeRange { TypeRange::Socket }

    fn begin(&self) -> u64 { self.begin }

    fn end(&self) -> u64 { self.end  }

    fn set_end(&mut self, val: u64) { self.end = val; }

    fn clone_dyn(&self) -> Box<dyn Range> { Box::new(self.clone()) }

}


#[cfg(test)]
mod test_types {

    use super::*;

    #[test]
    fn test_range_port() {

        let begin: u64 = 0;
        let end: u64 = 1000;

        let port: Port = Port::new(begin, end);

        assert_eq!(port.begin, begin);
        assert_eq!(port.end, end);
        assert_eq!(port.min(), 0);
        assert_eq!(port.max(), MAX_PORT);
        assert_eq!(port.range_of(), TypeRange::Port );
        assert_eq!(format!("{:?}", port), format!("({begin}, {end})"));

    }

    #[test]
    fn test_range_ip() {

        let begin: u64 = 0;
        let end: u64 = 1000;

        let ip: IP = IP::new(begin, end);

        assert_eq!(ip.begin, begin);
        assert_eq!(ip.end, end);
        assert_eq!(ip.min(), 0);
        assert_eq!(ip.max(), MAX_IP);
        assert_eq!(ip.range_of(), TypeRange::IP );
        assert_eq!(format!("{:?}", ip), format!("({begin}, {end})"));

    }

    #[test]
    fn test_range_socket() {

        let begin: u64 = 0;
        let end: u64 = 1000;

        let socket: Socket = Socket::new(begin, end);

        assert_eq!(socket.begin, begin);
        assert_eq!(socket.end, end);
        assert_eq!(socket.min(), 0);
        assert_eq!(socket.max(), MAX_SOCKET);
        assert_eq!(socket.range_of(), TypeRange::Socket );
        assert_eq!(format!("{:?}", socket), format!("({begin}, {end})"));

    }

}
