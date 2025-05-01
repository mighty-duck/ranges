
use std::time::Instant;

pub mod ranges;
use crate::ranges::{mask::Mask, types::{Port, IP, Socket}, logic::Op};

macro_rules! measure_time {
    ($code:block) => {{
        let start = Instant::now();
        let result = $code;
        let duration = start.elapsed();
        duration.as_secs_f64()
    }};
}



fn equal_tests() {

    let range_1a: Socket = Socket::new(0, 1000);
    let range_1b: Socket = Socket::new(1500, 2000);
    let range_2a: Socket = Socket::new(0, 1000);
    let range_2b: Socket = Socket::new(1100, 2000);

    let range_1 = Op {ranges: vec![Box::new(range_1a), Box::new(range_1b)], range_of: None};
    let range_2 = Op {ranges: vec![Box::new(range_2a), Box::new(range_2b)], range_of: None};


    println!("invert");
    println!("{}", range_1.clone());
    println!("{}", !range_1.clone());
    println!("logical and");
    println!("{}", range_1.clone());
    println!("{}", range_2.clone());
    println!("{}", range_1.clone() & range_2.clone());
    println!("logical or");
    println!("{}", range_1.clone());
    println!("{}", range_2.clone());
    println!("{}", range_1.clone() | range_2.clone());
    println!("logical sub var 1");
    println!("{}", range_1.clone());
    println!("{}", range_2.clone());
    println!("{}", range_1.clone() - range_2.clone());
    println!("logical sub var 2");
    println!("{}", range_1.clone());
    println!("{}", range_2.clone());
    println!("{}", range_2.clone() - range_1.clone());
    println!("logical xor");
    println!("{}", range_1.clone());
    println!("{}", range_2.clone());
    println!("{}", range_1.clone() ^ range_2.clone());

}


fn equal_mask_tests() {

    let mask_1 = Mask{
        range_ips: vec![IP::new(0, 0)],
        range_ports: vec![Port::new(0, 1000), Port::new(1500, 2000)],
    };

    let mask_2 = Mask{
        range_ips: vec![IP::new(0, 0)],
        range_ports: vec![Port::new(0, 1000), Port::new(1100, 2000)],
    };

    println!("collapsed mask 1");
    println!("{}", mask_1.to_collapsed().to_string());
    println!("collapsed mask 2");
    println!("{}", mask_2.to_collapsed().to_string());
    println!("invert mask 1");
    println!("{}", !mask_1.to_collapsed());
    println!("invert mask 2");
    println!("{}", !mask_2.to_collapsed());
    println!("or masks");
    println!("{}", mask_1.to_collapsed() | mask_2.to_collapsed());
    println!("and masks");
    println!("{}", mask_1.to_collapsed() & mask_2.to_collapsed());
    println!("\n");

}

fn perf_tests(power: i32) -> String {

    // println!("perf ip power {}", power);
    let count = 1 << power;

    let rules_1 = Mask{
        range_ips: vec![IP::new(0, count)],
        range_ports: vec![Port::new(0, 1500)],
    };

    let rules_2 = Mask{
        range_ips: vec![IP::new(0, count)],
        range_ports: vec![Port::new(0, 1000)],
    };

    println!("performance tests started for {count} ranges");

    let total_logic_or = measure_time!({
        _ = rules_1.clone().to_collapsed() | rules_2.clone().to_collapsed()
    });

    println!("logic or: {:?}", total_logic_or);

    let total_logic_and = measure_time!({
        _ = rules_1.clone().to_collapsed() & rules_2.clone().to_collapsed()
    });

    println!("logic and: {:?}", total_logic_and);

    let total_logic_xor = measure_time!({
        _ = rules_1.clone().to_collapsed() ^ rules_2.clone().to_collapsed()
    });

    println!("logic xor: {:?}", total_logic_xor);

    let total_logic_sub_1 = measure_time!({
        _ = rules_1.clone().to_collapsed() - rules_2.clone().to_collapsed()
    });

    println!("logic sub 1: {:?}", total_logic_sub_1);

    let total_logic_sub_2 = measure_time!({
        _ = rules_2.clone().to_collapsed() - rules_1.clone().to_collapsed()
    });

    println!("logic sub 2: {:?}", total_logic_sub_2);

    format!(
        "{count}\t{:.5}\t{:.5}\t{:.5}\t{:.5}\t{:.5}",
        total_logic_or,
        total_logic_and,
        total_logic_xor,
        total_logic_sub_1,
        total_logic_sub_2,
    )
}


fn main() {

    equal_tests();
    equal_mask_tests();

    let mut results: Vec<String> = vec![];

    for i in 0..21 {
        results.push(perf_tests(i));
    }

    println!("Benchmark complete");

    println!("count\tor\tand\txor\tsub 1\tsub 2");
    for item in results {
        println!("{}", item);
    }

}
