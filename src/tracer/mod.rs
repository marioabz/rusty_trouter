use std::io::{self, Write};
use dns_lookup::{ lookup_host, lookup_addr };
use std::{str, thread, string};
use icmp;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::time::Duration;

const LOWER_LIMIT_PORT: u32 = 33_434;
const UPPER_LIMIT_PORT: u32 = 33_534;
const UDP_PORT: u16 = 50_000;
const LOCAL_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const LOCAL_SOCK_ADDRS: SocketAddr = SocketAddr::new(LOCAL_IP, UDP_PORT);
const MSG: &[u8] = "Hello listeners".as_bytes();

enum MsgSignals {
    BEGGINIG,
    
}

pub struct TRouter {
    host: String,
    host_ip: IpAddr,
    remote_port: Option<u32>,
}


impl TRouter {
    pub fn new(host: &str) -> Self {
        let ips: Vec<IpAddr> = lookup_host(host).unwrap();
        let ip = ips[0];
        TRouter {
            host: host.to_string(),
            remote_port: None::<u32>,
            host_ip: ip
        }
    }

    fn init_message(&self) -> String {
        format!(
            "Traceroute to {0} ({1}), 64 hops max, 32 byte packets",
            self.host, self.host_ip
        )
    }

    fn set_port(&mut self, socket: &UdpSocket) {
        let mut host_port: String;
        for port in LOWER_LIMIT_PORT..UPPER_LIMIT_PORT {
            host_port = get_host_and_port(&self.host, port);
            &socket.connect(&host_port);
            match &socket.send(MSG) {
                Ok(_) => {
                    self.remote_port = Some(port);
                    break;
                },
                Err(_) => (),
            };
        }
        if self.remote_port.is_none() {
            panic!("Could not connect to remote host");
        }
    }
}


fn get_host_and_port(host: &str, port: u32) -> String {
    let host_ips = lookup_host(host).unwrap();
    let mut host_ip: Option<&IpAddr> = None::<&IpAddr>;
    for ip_value in host_ips.iter() {
        if let IpAddr::V4(_) = ip_value {
            host_ip = Some(ip_value);
        }
    }
    if host_ip.is_none() {
        panic!("No IPv4 address was found");
    }
    format!("{0}:{1}", host_ip.unwrap(), port)
}


fn msg_formater(host: &str, hop_ip: &str) {
    println!("{}", format!("{} ({})", host, hop_ip))
}


pub fn run_tracerouter(host: &str) {
    
    let mut ping_socket = icmp::IcmpSocket::connect(LOCAL_IP).unwrap();
    ping_socket.set_read_timeout(Some(Duration::new(1, 0)));
    let mut router = TRouter::new(host);
    router.init_message();
    let mut socket = UdpSocket::bind(LOCAL_SOCK_ADDRS).expect(
        "Couldn't connect to host"
    );
    let mut counter = 1;
    let mut inter_ip: String;

    // Loop that iterates over  64 hops MAX.
    for i in 1..64 {
        print!("{}\t", i);
        io::stdout().flush().unwrap();
        &socket.set_ttl(i).expect("Could not set TTL");
        if i == 1 {
            router.set_port(&mut socket);
        } else {
            // HANDLE ERRORS AND RETRY IT
            &socket.send(MSG);
        }
        let mut icmp_msg = [0u8; 20];

        // For loop that attempts 3 times to contact hop or final destination.
        for j in 0..3 {
            let recv_result = ping_socket.recv_from(&mut icmp_msg);
            if recv_result.is_err() {
                if j == 2 {
                    println!("*");
                    break;
                } else {
                    print!("*  ");
                    io::stdout().flush().unwrap();
                }
                continue
            }
            let (num_nutes, hop_ip) = recv_result.unwrap();
            match lookup_addr(&hop_ip) {
                Ok(hop_name) => {inter_ip=hop_name.to_string()},
                Err(e) => {inter_ip=hop_ip.to_string()}
            };
            msg_formater(&inter_ip, &hop_ip.to_string());
            break;
        }
        counter += 1;
    }   
}
