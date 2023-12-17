use dns_lookup::{ lookup_host, lookup_addr };
use std::{str, thread};
use icmp;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

const LOWER_LIMIT_PORT: u32 = 33_434;
const UPPER_LIMIT_PORT: u32 = 33_534;
const UDP_PORT: u16 = 50_000;
const LOCAL_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const LOCAL_SOCK_ADDRS: SocketAddr = SocketAddr::new(LOCAL_IP, UDP_PORT);
const MSG: &[u8] = "Hello listeners".as_bytes();


fn get_host_and_port(host: &str, port: u32) -> String {
    let host_ip = lookup_host(host).unwrap()[0];
    format!("{0}:{1}", host_ip, port)
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


fn msg_formater(hop_num: u32, host: &str, hop_ip: &str) {
    println!("{}", format!("{0}\t{1} ({2})", hop_num, host, hop_ip))
}


pub fn run_tracerouter(host: &str) {

    thread::spawn(|| {
        let mut counter = 1;
        let mut ping_socket = icmp::IcmpSocket::connect(LOCAL_IP).unwrap();
        let mut inter_ip: String;
        loop {
            let mut icmp_msg = [0u8; 20];
            let (num_nutes, hop_ip) = ping_socket.recv_from(&mut icmp_msg).unwrap();
            match lookup_addr(&hop_ip) {
                Ok(hop_name) => {inter_ip=hop_name.to_string()},
                Err(e) => {inter_ip=hop_ip.to_string()}
            };
            msg_formater(counter, &inter_ip, &hop_ip.to_string());
            thread::sleep(Duration::from_millis(100));
            counter += 1
        }
    });

    let mut router = TRouter::new(host);
    router.init_message();
    let mut socket = UdpSocket::bind(LOCAL_SOCK_ADDRS).expect(
        "Couldn't connect to host"
    );
    socket.set_ttl(1).expect("Couldn't set TTL");
    router.set_port(&mut socket);
    for i in 2..64 {
        socket.set_ttl(i).expect("Could not set TTL");
        &socket.send(MSG);
        thread::sleep(Duration::from_millis(50));
    }   
}
