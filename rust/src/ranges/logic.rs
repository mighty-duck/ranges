use std::cmp::{min, max};

use std::fmt::{Display, Formatter, Result};
use std::ops::{BitAnd, BitOr, BitXor, Not, Sub};

use super::types::{Port, IP, Socket, Range, TypeRange};


#[derive(Clone)]
pub struct Op {
    pub ranges: Vec<Box<dyn Range>>,
    pub range_of: Option<TypeRange>,
}


impl Display for Op {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            format!(
                "[{}]",
                self.ranges.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        )

    }
}


impl Op {

    fn get_range_of(&self) -> TypeRange {

        if self.range_of != None {
            self.range_of.clone().unwrap()
        } else {
            self.ranges[0].range_of()
        }

    }

}


fn build_from(range_of: TypeRange, begin: u64, end: u64) -> Box<dyn Range> {

    match range_of {
        TypeRange::Port => Box::new(Port{begin, end}),
        TypeRange::IP => Box::new(IP{begin, end} ),
        TypeRange::Socket => Box::new(Socket{begin, end}),
    }

}


impl Not for Op {

    type Output = Self;

    fn not(self) -> Self::Output {

        let mut ranges: Vec<Box<dyn Range>> = vec![];
        let range_of = self.get_range_of();

        if self.ranges[0].begin() != 0 {
            let begin: u64 = 0;
            let end: u64 = self.ranges[0].begin() - 1;
            ranges.push(build_from(range_of.clone(), begin, end));
        }

        for i in 0..self.ranges.len() - 1 {

            let current = &self.ranges[i];
            let next = &self.ranges[i + 1];

            let begin = current.end() + 1;
            let end = next.begin() - 1;

            if begin <= end {
                ranges.push(build_from(range_of.clone(), begin, end));
            }
        }

        let max_ip_num = self.ranges.last().unwrap().max();
        if self.ranges.last().unwrap().end() != max_ip_num {
            let begin: u64 = self.ranges.last().unwrap().end() + 1;
            let end: u64 = max_ip_num;
            ranges.push(build_from(range_of.clone(), begin, end));
        }

        Op {ranges, range_of: Some(range_of)}

    }

}


impl BitOr for Op {

    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {

        let range_of = self.get_range_of();

        let mut intervals = self.ranges;
        intervals.extend(rhs.ranges);

        intervals.sort_by(|a, b| a.begin().cmp(&b.begin()));

        let mut ranges: Vec<Box<dyn Range>> = vec![];
        ranges.push(intervals.first().unwrap().clone_dyn());

        for i in intervals.iter_mut().skip(1) {

            if (
                ranges.iter().last().unwrap().begin() <= i.begin()
                    &&
                i.begin() <= ranges.iter().last().unwrap().end()
            ) || (ranges.iter().last().unwrap().end() + 1 == i.end())    {
                let end: u64 = max(ranges.iter().last().unwrap().end(), i.end());
                ranges.iter_mut().last().unwrap().set_end(end);
            } else {
                ranges.push(build_from(range_of.clone(), i.begin(), i.end()));
            }

        }

        Op {ranges, range_of: Some(range_of)}

    }

}


impl BitAnd for Op {

    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {

        let mut ranges: Vec<Box<dyn Range>> = vec![];
        let range_of = self.get_range_of();

        let mut l_cnt: usize = 0;
        let mut r_cnt: usize = 0;

        while l_cnt < self.ranges.len() && r_cnt < rhs.ranges.len() {

            if rhs.ranges[r_cnt].begin() <= self.ranges[l_cnt].end()
                 &&
               self.ranges[l_cnt].begin() <= rhs.ranges[r_cnt].end()
            {

                let left: u64 = max(self.ranges[l_cnt].begin(), rhs.ranges[r_cnt].begin());
                let right: u64 = min(self.ranges[l_cnt].end(), rhs.ranges[r_cnt].end());
                ranges.push(build_from(range_of.clone(), left, right));
            }

            if self.ranges[l_cnt].end() > rhs.ranges[r_cnt].end() {
                r_cnt += 1;
            } else {
                l_cnt += 1;
            }

        }

        Op {ranges, range_of: Some(range_of)}

    }

}


impl Sub for Op {

    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output { self & !rhs }

}


impl BitXor for Op {

    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {

        let clone_self = Op {ranges: self.ranges.clone(), range_of: self.range_of.clone()};
        let clone_rhs = Op {ranges: rhs.ranges.clone(), range_of: rhs.range_of.clone()};

        self - rhs | clone_rhs - clone_self

    }

}



#[cfg(test)]
mod tests {

    use super::*;
    use super::super::types::{MAX_IP, MAX_PORT, MAX_SOCKET};

    #[test]
    fn test_operator_port_xor() {

        let a: Port = Port::new(0, 1000);
        let b: Port = Port::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Port = Port::new(0, 1000);
        let d: Port = Port::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let op_xor = op_1 ^ op_2;
        assert_eq!(op_xor.ranges.len(), 1);
        assert_eq!(op_xor.ranges[0].begin(), 1100);
        assert_eq!(op_xor.ranges[0].end(), 1499);
        assert_eq!(op_xor.range_of.unwrap(), TypeRange::Port);

    }

    #[test]
    fn test_operator_ip_xor() {

        let a: IP = IP::new(0, 1000);
        let b: IP = IP::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: IP = IP::new(0, 1000);
        let d: IP = IP::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let op_xor = op_1 ^ op_2;
        assert_eq!(op_xor.ranges.len(), 1);
        assert_eq!(op_xor.ranges[0].begin(), 1100);
        assert_eq!(op_xor.ranges[0].end(), 1499);
        assert_eq!(op_xor.range_of.unwrap(), TypeRange::IP);

    }

    #[test]
    fn test_operator_socket_xor() {

        let a: Socket = Socket::new(0, 1000);
        let b: Socket = Socket::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Socket = Socket::new(0, 1000);
        let d: Socket = Socket::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let op_xor = op_1 ^ op_2;
        assert_eq!(op_xor.ranges.len(), 1);
        assert_eq!(op_xor.ranges[0].begin(), 1100);
        assert_eq!(op_xor.ranges[0].end(), 1499);
        assert_eq!(op_xor.range_of.unwrap(), TypeRange::Socket);

    }

    #[test]
    fn test_operator_port_sub_1() {

        let a: Port = Port::new(0, 1000);
        let b: Port = Port::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Port = Port::new(0, 1000);
        let d: Port = Port::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let op_sub = op_1 - op_2;
        assert_eq!(op_sub.ranges.len(), 0);
        assert_eq!(op_sub.range_of.unwrap(), TypeRange::Port);

    }

    #[test]
    fn test_operator_port_sub_2() {

        let a: Port = Port::new(0, 1000);
        let b: Port = Port::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Port = Port::new(0, 1000);
        let d: Port = Port::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let op_sub = op_2 - op_1;
        assert_eq!(op_sub.ranges.len(), 1);
        assert_eq!(op_sub.ranges[0].begin(), 1100);
        assert_eq!(op_sub.ranges[0].end(), 1499);
        assert_eq!(op_sub.range_of.unwrap(), TypeRange::Port);

    }

    #[test]
    fn test_operator_ip_sub_1() {

        let a: IP = IP::new(0, 1000);
        let b: IP = IP::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: IP = IP::new(0, 1000);
        let d: IP = IP::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let op_sub = op_1 - op_2;
        assert_eq!(op_sub.ranges.len(), 0);
        assert_eq!(op_sub.range_of.unwrap(), TypeRange::IP);

    }

    #[test]
    fn test_operator_ip_sub_2() {

        let a: IP = IP::new(0, 1000);
        let b: IP = IP::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: IP = IP::new(0, 1000);
        let d: IP = IP::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let op_sub = op_2 - op_1;
        assert_eq!(op_sub.ranges.len(), 1);
        assert_eq!(op_sub.ranges[0].begin(), 1100);
        assert_eq!(op_sub.ranges[0].end(), 1499);
        assert_eq!(op_sub.range_of.unwrap(), TypeRange::IP);

    }

    #[test]
    fn test_operator_socket_sub_1() {

        let a: Socket = Socket::new(0, 1000);
        let b: Socket = Socket::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Socket = Socket::new(0, 1000);
        let d: Socket = Socket::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let op_sub = op_1 - op_2;
        assert_eq!(op_sub.ranges.len(), 0);
        assert_eq!(op_sub.range_of.unwrap(), TypeRange::Socket);

    }

    #[test]
    fn test_operator_socket_sub_2() {

        let a: Socket = Socket::new(0, 1000);
        let b: Socket = Socket::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Socket = Socket::new(0, 1000);
        let d: Socket = Socket::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let op_sub = op_2 - op_1;
        assert_eq!(op_sub.ranges.len(), 1);
        assert_eq!(op_sub.ranges[0].begin(), 1100);
        assert_eq!(op_sub.ranges[0].end(), 1499);
        assert_eq!(op_sub.range_of.unwrap(), TypeRange::Socket);

    }

    #[test]
    fn test_operator_port_and() {

        let a: Port = Port::new(0, 1000);
        let b: Port = Port::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Port = Port::new(0, 1000);
        let d: Port = Port::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let log_and = op_1 & op_2;
        assert_eq!(log_and.ranges.len(), 2);
        assert_eq!(log_and.ranges[0].begin(), 0);
        assert_eq!(log_and.ranges[0].end(), 1000);
        assert_eq!(log_and.ranges[1].begin(), 1500);
        assert_eq!(log_and.ranges[1].end(), 2000);
        assert_eq!(log_and.range_of.unwrap(), TypeRange::Port);

    }

    #[test]
    fn test_operator_ip_and() {

        let a: IP = IP::new(0, 1000);
        let b: IP = IP::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: IP = IP::new(0, 1000);
        let d: IP = IP::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let log_and = op_1 & op_2;
        assert_eq!(log_and.ranges.len(), 2);
        assert_eq!(log_and.ranges[0].begin(), 0);
        assert_eq!(log_and.ranges[0].end(), 1000);
        assert_eq!(log_and.ranges[1].begin(), 1500);
        assert_eq!(log_and.ranges[1].end(), 2000);
        assert_eq!(log_and.range_of.unwrap(), TypeRange::IP);

    }

    #[test]
    fn test_operator_socket_and() {

        let a: Socket = Socket::new(0, 1000);
        let b: Socket = Socket::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Socket = Socket::new(0, 1000);
        let d: Socket = Socket::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let log_and = op_1 & op_2;
        assert_eq!(log_and.ranges.len(), 2);
        assert_eq!(log_and.ranges[0].begin(), 0);
        assert_eq!(log_and.ranges[0].end(), 1000);
        assert_eq!(log_and.ranges[1].begin(), 1500);
        assert_eq!(log_and.ranges[1].end(), 2000);
        assert_eq!(log_and.range_of.unwrap(), TypeRange::Socket);

    }

    #[test]
    fn test_operator_port_or() {

        let a: Port = Port::new(0, 1000);
        let b: Port = Port::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Port = Port::new(0, 1000);
        let d: Port = Port::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let log_or = op_1 | op_2;
        assert_eq!(log_or.ranges.len(), 2);
        assert_eq!(log_or.ranges[0].begin(), 0);
        assert_eq!(log_or.ranges[0].end(), 1000);
        assert_eq!(log_or.ranges[1].begin(), 1100);
        assert_eq!(log_or.ranges[1].end(), 2000);
        assert_eq!(log_or.range_of.unwrap(), TypeRange::Port);

    }

    #[test]
    fn test_operator_ip_or() {

        let a: IP = IP::new(0, 1000);
        let b: IP = IP::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: IP = IP::new(0, 1000);
        let d: IP = IP::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let log_or = op_1 | op_2;
        assert_eq!(log_or.ranges.len(), 2);
        assert_eq!(log_or.ranges[0].begin(), 0);
        assert_eq!(log_or.ranges[0].end(), 1000);
        assert_eq!(log_or.ranges[1].begin(), 1100);
        assert_eq!(log_or.ranges[1].end(), 2000);
        assert_eq!(log_or.range_of.unwrap(), TypeRange::IP);

    }

    #[test]
    fn test_operator_socket_or() {

        let a: Socket = Socket::new(0, 1000);
        let b: Socket = Socket::new(1500, 2000);
        let op_1 = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        let c: Socket = Socket::new(0, 1000);
        let d: Socket = Socket::new(1100, 2000);
        let op_2 = Op {ranges: vec![Box::new(c), Box::new(d)], range_of: None};

        let log_or = op_1 | op_2;
        assert_eq!(log_or.ranges.len(), 2);
        assert_eq!(log_or.ranges[0].begin(), 0);
        assert_eq!(log_or.ranges[0].end(), 1000);
        assert_eq!(log_or.ranges[1].begin(), 1100);
        assert_eq!(log_or.ranges[1].end(), 2000);
        assert_eq!(log_or.range_of.unwrap(), TypeRange::Socket);

    }

    #[test]
    fn test_operator_port_invert_at_null() {

        let a: Port = Port::new(0, 2);
        let b: Port = Port::new(1, 3);

        let op = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        assert_eq!(op.ranges.len(), 2);
        assert_eq!(op.range_of, None);

        let log_inv = !op;
        assert_eq!(log_inv.ranges.len(), 1);
        assert_eq!(log_inv.ranges[0].begin(), 4);
        assert_eq!(log_inv.ranges[0].end(), MAX_PORT);
        assert_eq!(log_inv.range_of.unwrap(), TypeRange::Port);

    }

    #[test]
    fn test_operator_invert_port_at_not_null() {

        let a: Port = Port::new(10, 20);
        let b: Port = Port::new(15, 30);
        let c: Port = Port::new(20, 40);

        let op = Op {ranges: vec![Box::new(a), Box::new(b), Box::new(c)], range_of: None};
        assert_eq!(op.ranges.len(), 3);
        assert_eq!(op.range_of, None);

        let log_inv = !op;
        assert_eq!(log_inv.ranges.len(), 2);
        assert_eq!(log_inv.ranges[0].begin(), 0);
        assert_eq!(log_inv.ranges[0].end(), 9);
        assert_eq!(log_inv.range_of.unwrap(), TypeRange::Port);

    }

    #[test]
    fn test_operator_ip_invert_at_null() {

        let a: IP = IP::new(0, 2);
        let b: IP = IP::new(1, 3);

        let op = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        assert_eq!(op.ranges.len(), 2);
        assert_eq!(op.range_of, None);

        let log_inv = !op;
        assert_eq!(log_inv.ranges.len(), 1);
        assert_eq!(log_inv.ranges[0].begin(), 4);
        assert_eq!(log_inv.ranges[0].end(), MAX_IP);
        assert_eq!(log_inv.range_of.unwrap(), TypeRange::IP);

    }

    #[test]
    fn test_operator_invert_ip_at_not_null() {

        let a: IP = IP::new(10, 20);
        let b: IP = IP::new(15, 30);
        let c: IP = IP::new(20, 40);

        let op = Op {ranges: vec![Box::new(a), Box::new(b), Box::new(c)], range_of: None};
        assert_eq!(op.ranges.len(), 3);
        assert_eq!(op.range_of, None);

        let log_inv = !op;
        assert_eq!(log_inv.ranges.len(), 2);
        assert_eq!(log_inv.ranges[0].begin(), 0);
        assert_eq!(log_inv.ranges[0].end(), 9);
        assert_eq!(log_inv.range_of.unwrap(), TypeRange::IP);

    }

    #[test]
    fn test_operator_socket_invert_at_null() {

        let a: Socket = Socket::new(0, 2);
        let b: Socket = Socket::new(1, 3);

        let op = Op {ranges: vec![Box::new(a), Box::new(b)], range_of: None};
        assert_eq!(op.ranges.len(), 2);
        assert_eq!(op.range_of, None);

        let log_inv = !op;
        assert_eq!(log_inv.ranges.len(), 1);
        assert_eq!(log_inv.ranges[0].begin(), 4);
        assert_eq!(log_inv.ranges[0].end(), MAX_SOCKET);
        assert_eq!(log_inv.range_of.unwrap(), TypeRange::Socket);

    }

    #[test]
    fn test_operator_socket_ip_at_not_null() {

        let a: Socket = Socket::new(10, 20);
        let b: Socket = Socket::new(15, 30);
        let c: Socket = Socket::new(20, 40);

        let op = Op {ranges: vec![Box::new(a), Box::new(b), Box::new(c)], range_of: None};
        assert_eq!(op.ranges.len(), 3);
        assert_eq!(op.range_of, None);

        let log_inv = !op;
        assert_eq!(log_inv.ranges.len(), 2);
        assert_eq!(log_inv.ranges[0].begin(), 0);
        assert_eq!(log_inv.ranges[0].end(), 9);
        assert_eq!(log_inv.range_of.unwrap(), TypeRange::Socket);

    }

}
