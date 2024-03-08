extern crate bitcoin;
extern crate num_bigint;

use bitcoin::key::PrivateKey;
use bitcoin::network::Network;
use bitcoin::{address::Address, PublicKey};
use chrono::prelude::*;
use num::One;
use num_bigint::BigInt;
use std::io;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

fn main() {
    let mut input_str = String::new();
    let cpus: u32 = num_cpus::get() as u32;

    println!("[+] POC BTC Puzzle Solver [+]\n");
    let mut puzzle_no: u128 = num::zero();
    while puzzle_no < 1 || puzzle_no > 160 {
        println!("Enter puzzle # (1-160):");
        io::stdin()
            .read_line(&mut input_str)
            .expect("Failed to read line");
        puzzle_no = input_str
            .trim()
            .parse()
            .expect("Please enter a valid number");
        input_str.clear();
    }

    println!("Enter target address:");
    io::stdin()
        .read_line(&mut input_str)
        .expect("Failed to read line");
    let target_address: String = input_str.trim().to_string();
    input_str.clear();

    let mut threads: u32 = num::zero();
    while threads < 1 || threads > cpus {
        println!("Enter # of threads (1-{}):", cpus);
        io::stdin()
            .read_line(&mut input_str)
            .expect("Failed to read line");

        threads = input_str
            .trim()
            .parse()
            .expect("Please enter a valid number");
        input_str.clear();
    }

    let range_start: BigInt = num::pow(BigInt::from(2), (puzzle_no - 1) as usize);
    let range_end: BigInt = num::pow(BigInt::from(2), puzzle_no as usize) - BigInt::one();

    let found_flag = Arc::new(Mutex::new(false));
    let pool = ThreadPool::new(threads as usize);

    let time_start = Local::now();
    println!(
        "\n[+] Search started at {}",
        time_start.format("%Y-%m-%d %H:%M:%S")
    );

    let total_range = range_end.clone() - &range_start + BigInt::one(); // Dereference the Arc to access BigInt
    let range_size = &total_range / threads;

    for i in 0..threads {
        let target_address = target_address.clone();
        let pool = pool.clone();
        let found_flag = found_flag.clone();
        let range_start = range_start.clone(); // Clone the range start value

        let start = range_start.clone() + (i * range_size.clone());
        let end = if i == threads - 1 {
            range_end.clone()
        } else {
            let next_start = start.clone() + range_size.clone();
            let next_end = if next_start > range_end {
                range_end.clone()
            } else {
                next_start - BigInt::one()
            };
            next_end
        };

        let execute_job = move || {
            search(
                start.clone(),
                end.clone(),
                target_address,
                found_flag.clone(),
            );
        };
        pool.execute(execute_job);
    }

    pool.join();
}

fn search(
    range_start: BigInt,
    range_end: BigInt,
    target_address: String,
    found_flag: Arc<Mutex<bool>>,
) {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let mut current_key: BigInt = range_start;
    loop {
        let mut found_flag = found_flag.lock().unwrap();
        if current_key > range_end || *found_flag {
            break;
        }
        let private_key_hex: String = format!("{:0>64x}", current_key);
        let private_key_bytes: Vec<u8> =
            hex::decode(&private_key_hex).expect("Failed to decode private key");

        let private_key: PrivateKey = PrivateKey {
            compressed: true,
            network: Network::Bitcoin,
            inner: bitcoin::secp256k1::SecretKey::from_slice(&private_key_bytes).unwrap(),
        };

        let public_key: PublicKey = private_key.public_key(&secp);
        let address: String = Address::p2pkh(&public_key, Network::Bitcoin).to_string();
        print!("\r[+] WIF: {}", private_key);
        if address == target_address {
            let line_of_dashes = "-".repeat(80);
            let time_current = Local::now();
            println!(
                "\n[+] {}\n[+] KEY FOUND! {}\n[+] Decimal: {} \n[+] Private Key: {} \n[+] Public Key: {} \n[+] Address: {} \n[+] {}\n",
                line_of_dashes,
                time_current.format("%Y-%m-%d %H:%M:%S"),
                current_key,
                private_key,
                public_key,
                address,
                line_of_dashes
            );
            *found_flag = true;
        }
        current_key += BigInt::one();
    }
}
