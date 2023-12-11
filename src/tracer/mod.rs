use dns_lookup::lookup_host;
use icmp;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};

pub struct TRouter {
    host: String,
    counter: u8,
}

impl TRouter {
    pub fn new(host: &str) -> Self {
        TRouter {
            host: host.to_string(),
            counter: 1,
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
}
