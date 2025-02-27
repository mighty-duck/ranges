
use crate::ranges::types::{Port, IP, Socket, TypeRange, Range};
use crate::ranges::logic::{Op};


#[derive(Clone)]
pub struct Mask {
    pub range_ips: Vec<IP>,
    pub range_ports: Vec<Port>,
}



pub fn convert(a: u64, b: u64) -> u64 {

    let a_bytes = (a as u32).to_be_bytes();
    let b_bytes = (b as u16).to_be_bytes();

    let buff = [a_bytes.as_slice(), b_bytes.as_slice()].concat();

    u64::from_be_bytes([
        0,
        0,
        buff[0],
        buff[1],
        buff[2],
        buff[3],
        buff[4],
        buff[5],
    ])

}


impl Mask {

    pub fn to_collapsed(&self) -> Op {

        let mut ranges: Vec<Box<dyn Range>> = vec![];

        for addr in &self.range_ips {
            for ip in addr.begin..addr.end + 1 {
                for port_range in &self.range_ports {
                    let begin = convert(ip, port_range.begin);
                    let end = convert(ip, port_range.end);

                    let socket = Socket::new(begin, end);
                    ranges.push(Box::new(socket));
                }
            }
        }

        Op {ranges, range_of: Some(TypeRange::IP)}

    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert() {
        assert_eq!(convert(10, 20), 655380);
    }

}
