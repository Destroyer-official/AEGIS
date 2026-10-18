use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    Unknown(u8),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("packet too short: need at least {0} bytes, got {1}")]
    TooShort(usize, usize),
    #[error("unsupported ethertype: 0x{0:04x}")]
    UnsupportedEthertype(u16),
    #[error("invalid IP header")]
    InvalidIpHeader,
}

#[derive(Debug, Clone)]
pub struct ParsedPacket {
    pub src_mac: [u8; 6],
    pub dst_mac: [u8; 6],
    pub ethertype: u16,
    pub vlan_id: Option<u16>,
    pub src_ip: std::net::IpAddr,
    pub dst_ip: std::net::IpAddr,
    pub protocol: Protocol,
    pub src_port: u16,
    pub dst_port: u16,
    pub payload_len: usize,
    pub tcp_flags: Option<TcpFlags>,
    pub raw_len: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TcpFlags {
    pub syn: bool,
    pub ack: bool,
    pub fin: bool,
    pub rst: bool,
    pub psh: bool,
    pub urg: bool,
}

impl ParsedPacket {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 14 {
            return Err(ParseError::TooShort(14, data.len()));
        }

        let dst_mac = [data[0], data[1], data[2], data[3], data[4], data[5]];
        let src_mac = [data[6], data[7], data[8], data[9], data[10], data[11]];
        let mut ethertype = u16::from_be_bytes([data[12], data[13]]);
        let mut offset = 14;

        let vlan_id = if ethertype == 0x8100 {
            if data.len() < 18 {
                return Err(ParseError::TooShort(18, data.len()));
            }
            let vid = u16::from_be_bytes([data[14], data[15]]) & 0x0FFF;
            ethertype = u16::from_be_bytes([data[16], data[17]]);
            offset = 18;
            Some(vid)
        } else {
            None
        };

        match ethertype {
            0x0800 => Self::parse_ipv4(data, offset, src_mac, dst_mac, ethertype, vlan_id),
            0x86DD => Self::parse_ipv6(data, offset, src_mac, dst_mac, ethertype, vlan_id),
            _ => Err(ParseError::UnsupportedEthertype(ethertype)),
        }
    }

    fn parse_ipv4(
        data: &[u8],
        offset: usize,
        src_mac: [u8; 6],
        dst_mac: [u8; 6],
        ethertype: u16,
        vlan_id: Option<u16>,
    ) -> Result<Self, ParseError> {
        if data.len() < offset + 20 {
            return Err(ParseError::TooShort(offset + 20, data.len()));
        }

        let ip_start = offset;
        let ihl = (data[ip_start] & 0x0F) as usize * 4;
        let total_len = u16::from_be_bytes([data[ip_start + 2], data[ip_start + 3]]) as usize;

        let protocol = match data[ip_start + 9] {
            6 => Protocol::Tcp,
            17 => Protocol::Udp,
            1 => Protocol::Icmp,
            p => Protocol::Unknown(p),
        };

        let src_ip = Ipv4Addr::new(
            data[ip_start + 12],
            data[ip_start + 13],
            data[ip_start + 14],
            data[ip_start + 15],
        );
        let dst_ip = Ipv4Addr::new(
            data[ip_start + 16],
            data[ip_start + 17],
            data[ip_start + 18],
            data[ip_start + 19],
        );

        let transport_offset = ip_start + ihl;
        let (src_port, dst_port, tcp_flags, payload_len) = match protocol {
            Protocol::Tcp => {
                if data.len() < transport_offset + 20 {
                    return Err(ParseError::TooShort(transport_offset + 20, data.len()));
                }
                let sp = u16::from_be_bytes([data[transport_offset], data[transport_offset + 1]]);
                let dp =
                    u16::from_be_bytes([data[transport_offset + 2], data[transport_offset + 3]]);
                let flags = TcpFlags {
                    syn: (data[transport_offset + 13] & 0x02) != 0,
                    ack: (data[transport_offset + 13] & 0x10) != 0,
                    fin: (data[transport_offset + 13] & 0x01) != 0,
                    rst: (data[transport_offset + 13] & 0x04) != 0,
                    psh: (data[transport_offset + 13] & 0x08) != 0,
                    urg: (data[transport_offset + 13] & 0x20) != 0,
                };
                let tcp_hdr_len = ((data[transport_offset + 12] >> 4) & 0x0F) as usize * 4;
                let payload = total_len - ihl - tcp_hdr_len;
                (sp, dp, Some(flags), payload)
            }
            Protocol::Udp => {
                if data.len() < transport_offset + 8 {
                    return Err(ParseError::TooShort(transport_offset + 8, data.len()));
                }
                let sp = u16::from_be_bytes([data[transport_offset], data[transport_offset + 1]]);
                let dp =
                    u16::from_be_bytes([data[transport_offset + 2], data[transport_offset + 3]]);
                let udp_len =
                    u16::from_be_bytes([data[transport_offset + 4], data[transport_offset + 5]])
                        as usize;
                (sp, dp, None, udp_len - 8)
            }
            _ => (0, 0, None, total_len - ihl),
        };

        Ok(ParsedPacket {
            src_mac,
            dst_mac,
            ethertype,
            vlan_id,
            src_ip: std::net::IpAddr::V4(src_ip),
            dst_ip: std::net::IpAddr::V4(dst_ip),
            protocol,
            src_port,
            dst_port,
            payload_len,
            tcp_flags,
            raw_len: data.len(),
        })
    }

    fn parse_ipv6(
        data: &[u8],
        offset: usize,
        src_mac: [u8; 6],
        dst_mac: [u8; 6],
        ethertype: u16,
        vlan_id: Option<u16>,
    ) -> Result<Self, ParseError> {
        if data.len() < offset + 40 {
            return Err(ParseError::TooShort(offset + 40, data.len()));
        }

        let ip_start = offset;
        let payload_len = u16::from_be_bytes([data[ip_start + 4], data[ip_start + 5]]) as usize;
        let next_header = data[ip_start + 6];

        let protocol = match next_header {
            6 => Protocol::Tcp,
            17 => Protocol::Udp,
            58 => Protocol::Icmp,
            p => Protocol::Unknown(p),
        };

        let src_ip_bytes: [u8; 16] = data[ip_start + 8..ip_start + 24].try_into().unwrap();
        let dst_ip_bytes: [u8; 16] = data[ip_start + 24..ip_start + 40].try_into().unwrap();

        let transport_offset = ip_start + 40;
        let (src_port, dst_port, tcp_flags) = match protocol {
            Protocol::Tcp => {
                if data.len() < transport_offset + 20 {
                    return Err(ParseError::TooShort(transport_offset + 20, data.len()));
                }
                let sp = u16::from_be_bytes([data[transport_offset], data[transport_offset + 1]]);
                let dp =
                    u16::from_be_bytes([data[transport_offset + 2], data[transport_offset + 3]]);
                let flags = TcpFlags {
                    syn: (data[transport_offset + 13] & 0x02) != 0,
                    ack: (data[transport_offset + 13] & 0x10) != 0,
                    fin: (data[transport_offset + 13] & 0x01) != 0,
                    rst: (data[transport_offset + 13] & 0x04) != 0,
                    psh: (data[transport_offset + 13] & 0x08) != 0,
                    urg: (data[transport_offset + 13] & 0x20) != 0,
                };
                (sp, dp, Some(flags))
            }
            Protocol::Udp => {
                if data.len() < transport_offset + 8 {
                    return Err(ParseError::TooShort(transport_offset + 8, data.len()));
                }
                let sp = u16::from_be_bytes([data[transport_offset], data[transport_offset + 1]]);
                let dp =
                    u16::from_be_bytes([data[transport_offset + 2], data[transport_offset + 3]]);
                (sp, dp, None)
            }
            _ => (0, 0, None),
        };

        Ok(ParsedPacket {
            src_mac,
            dst_mac,
            ethertype,
            vlan_id,
            src_ip: std::net::IpAddr::V6(Ipv6Addr::from(src_ip_bytes)),
            dst_ip: std::net::IpAddr::V6(Ipv6Addr::from(dst_ip_bytes)),
            protocol,
            src_port,
            dst_port,
            payload_len,
            tcp_flags,
            raw_len: data.len(),
        })
    }
}
