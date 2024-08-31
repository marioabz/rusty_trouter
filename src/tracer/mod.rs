
use std::io::{self, Write,};
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
const MSG: &[u8] = "H".as_bytes();

// TO DO:
// Send 3 probes and calculate time of each probe
// If no response in 5 secs, a * is printed

pub struct TRouter {
    ips: Vec<IpAddr>,
    probing_socket: UdpSocket,
    icmp_socket: icmp::IcmpSocket,
    device: io::Stdout,
}

impl TRouter {
    pub fn new(host: &str) -> Self {
        let ips: Vec<IpAddr> = lookup_host(host).unwrap();
        let ip = &ips[0];
        let mut stdout = io::stdout();
        let init_msg = format!("Traceroute to {0} ({1}), 64 hops max, 32 byte packets\n", host, ip);
        stdout.write(init_msg.as_bytes());
        stdout.flush();
        if ips.len() > 1 {
            let msg = format!("warning: {host} has multiple ips, using {ip}\n");
            stdout.write(msg.as_bytes());
            stdout.flush();
        }
        let mut socket = UdpSocket::bind(LOCAL_SOCK_ADDRS).expect(
            "Couldn't connect to host"
        );
        let remote_port = TRouter::get_port(&ip, &socket);
        let mut ping_socket = icmp::IcmpSocket::connect(LOCAL_IP).unwrap();
        TRouter {
            ips: ips,
            probing_socket: socket,
            icmp_socket: ping_socket,
            device: stdout,
        }
    }

    fn get_port(ip: &IpAddr, socket: &UdpSocket) -> u32 {
        for port in LOWER_LIMIT_PORT..UPPER_LIMIT_PORT {
            let mut host_port = format!("{0}:{1}", ip, port);
            &socket.connect(&host_port);
            match &socket.send(MSG) {
                Ok(_) => {
                    return port
                },
                Err(_) => ()
            };
        }
        0
    }
}


fn msg_formater(host: &str, hop_ip: &str) -> String{
   format!("{} ({})\n", host, hop_ip)
}


pub fn run_tracerouter(host: &str) {
    
    let mut router = TRouter::new(host);
    router.icmp_socket.set_read_timeout(Some(Duration::new(1, 0)));
    let mut counter = 1;
    let mut inter_ip: String;

    // Loop that iterates over  64 hops MAX.
    for i in 1..64 {

        router.device.write(format!("{i}\t").as_bytes());
        router.device.flush();
        let _ = router.probing_socket.set_ttl(i).expect("Could not set TTL");

        // For loop that attempts 3 times to contact hop or final destination.
        for j in 0..3 {
            let mut icmp_msg = [0u8; 20];
            let _ = router.probing_socket.send(MSG);
            let recv_result = router.icmp_socket.recv_from(&mut icmp_msg);
            if recv_result.is_err() {
                if j == 2 {
                    router.device.write(String::from("*\n").as_bytes());
                    router.device.flush();
                    break;
                } else {
                    router.device.write(String::from("*  ").as_bytes());
                    io::stdout().flush().unwrap();
                }
                continue
            }
            let (num_nutes, hop_ip) = recv_result.unwrap();
            match lookup_addr(&hop_ip) {
                Ok(hop_name) => {inter_ip=hop_name.to_string()},
                Err(e) => {inter_ip=hop_ip.to_string()}
            };
            let mut suc_msg = msg_formater(&inter_ip, &hop_ip.to_string());
            router.device.write(&suc_msg.as_bytes());
            break;
        }
        counter += 1;
    }   
}
