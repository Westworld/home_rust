use std::net::{SocketAddr, UdpSocket};
use chrono::prelude::*;
use std::io::prelude::*;
use std::fs::OpenOptions;


pub fn do_udp() {
    {
    let addr = SocketAddr::from(([0, 0, 0, 0], 19814));
    let socket: UdpSocket;
    match  UdpSocket::bind(addr) {
        Ok(t) => {
            socket = t;
        }
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }        
    }

    let mut file;
    let path: &str;
    let hostname = env!("HOSTNAME");  // compile time!!!
    if hostname != "Thomas_test" {
         path = "user/thomas/udplog.txt";
    }
    else {
        path = "/home/pi/udplog.txt";
    }

    match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path){
            Ok(t) => {
                file = t;
            }
            Err(e) => {
                eprintln!("{:?}", e);
                return;
            }               
        }

    loop {

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 50];

        let amt: usize;
        let src: SocketAddr;

        match socket.recv_from(&mut buf) {
            Ok(t) => {
                (amt, src) = t;
            }
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            }            
        }

        let message: &str;
        match std::str::from_utf8(&buf[..amt]) {
            Ok(t) => {
                message = t;
            }
            Err(e) => {
                message = "";
                eprintln!("{:?}", e);
            }
        }

        let local: DateTime<Local> = Local::now();
        let dt_string = local.format("%d/%m/%Y %H:%M:%S");
        let log = format!("{}\t{}\t{}", dt_string, src, message);
        println!("{}", log);

        if let Err(e) = writeln!(file, "{}", log) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }

    } // the socket is closed here

}



