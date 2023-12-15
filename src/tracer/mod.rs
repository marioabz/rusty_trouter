use dns_lookup::lookup_host;
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
    counter: u8,
    local_port: u32,
    remote_port: Option<u32>,
}

impl TRouter {
    pub fn new(host: &str) -> Self {
        TRouter {
            host: host.to_string(),
            counter: 1,
            local_port: 50_00,
            remote_port: None::<u32>
        }
    }

    pub fn init_message(&mut self) -> String {
        let ips: Vec<IpAddr> = lookup_host(&self.host).unwrap();
        let ip = ips[0];
        format!(
            "Traceroute to {0} ({ip}), 64 hops max, 32 byte packets",
            self.host
        )
    }

    fn set_port(&mut self, socket: &UdpSocket) {
        let mut host_port: String;
        socket.set_ttl(1).expect(
            "Couldn't set TTL"
        );
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

    fn send_message(&mut self, socket: &UdpSocket) {
        for i in 2..65 {
            socket.set_ttl(i).expect(
                "Couldn't set TTL"
            );
            match &socket.send(MSG) {
                Ok(_) => (),
                Err(_) => (),
            };
            thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn run(&mut self) {
        thread::spawn(|| {
            let mut ping_socket = icmp::IcmpSocket::connect(LOCAL_IP).unwrap();
            loop {
                let mut icmp_msg = [0u8; 20];
                match ping_socket.recv_from(&mut icmp_msg) {
                    Ok(res) => println!("ICMP bytes read: {:?}", &(res.1)),
                    Err(e) => (),
                };
                thread::sleep(Duration::from_millis(100))
            }
        });
        let socket = UdpSocket::bind(LOCAL_SOCK_ADDRS).expect(
            "Couldn't connect to host"
        );
        self.set_port(&socket);
        self.send_message(&socket);
        thread::sleep(Duration::from_millis(10000))
    }
}
