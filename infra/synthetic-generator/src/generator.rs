use aegis_proto::*;
use rand::Rng;
use sha2::Digest;
use uuid::Uuid;

pub struct Generator {
    source_pop: String,
    rng: rand::rngs::ThreadRng,
}

impl Generator {
    pub fn new(source_pop: String) -> Self {
        Self {
            source_pop,
            rng: rand::thread_rng(),
        }
    }

    /// Generate a synthetic L4Response for the raw-l4-scans topic
    pub fn generate_l4_response(&mut self) -> L4Response {
        let ip = self.random_ip_bytes();
        let port = self.random_port();
        let now = chrono::Utc::now();

        L4Response {
            event_id: Uuid::new_v4().to_string(),
            target_ip: ip,
            target_port: port,
            transport: Transport::Tcp as i32,
            result: self.random_l4_result(),
            rtt_usec: self.rng.gen_range(100..50000),
            ttl: self.rng.gen_range(32..255),
            tcp_window: self.rng.gen_range(1024..65535),
            source_pop: self.source_pop.clone(),
            sent_at: (now - chrono::Duration::milliseconds(self.rng.gen_range(1..100)))
                .to_rfc3339(),
            received_at: now.to_rfc3339(),
        }
    }

    /// Generate a synthetic TLSFingerprint for the tls-fingerprints topic
    pub fn generate_tls_fingerprint(&mut self) -> TLSFingerprint {
        let ip = self.random_ip_bytes();
        let port = self.random_tls_port();
        let now = chrono::Utc::now();

        let cipher_hash = self.random_hex(12);
        let ext_hash = self.random_hex(12);
        let cipher_count = self.rng.gen_range(10..20);
        let ext_count = self.rng.gen_range(5..15);

        TLSFingerprint {
            event_id: Uuid::new_v4().to_string(),
            target_ip: ip,
            target_port: port,
            transport: Transport::Tcp as i32,
            ja4: format!(
                "t13d{:02}{:02}h2_{}_{}",
                cipher_count, ext_count, cipher_hash, ext_hash
            ),
            ja4s: format!("s13{}_{}", self.random_hex(12), self.random_hex(12)),
            tls_version: "13".into(),
            cipher_suite: "TLS_AES_256_GCM_SHA384".into(),
            sni: format!("*.{}", self.random_domain()),
            alpn: "h2".into(),
            certificate_chain: String::new(),
            cert_fingerprint: self.random_hex(64),
            cert_subject_cn: format!("*.{}", self.random_domain()),
            cert_issuer_cn: format!("{} CA", self.random_org()),
            cert_sans: vec![
                format!("*.{}", self.random_domain()),
                format!("*.{}", self.random_domain()),
            ],
            source_pop: self.source_pop.clone(),
            observed_at: now.to_rfc3339(),
        }
    }

    /// Generate a synthetic L7Banner for the l7-banners topic
    pub fn generate_l7_banner(&mut self) -> L7Banner {
        let ip = self.random_ip_bytes();
        let port = self.random_port();
        let now = chrono::Utc::now();

        let (protocol, detection, banner, http_method, http_status, ssh_version) = match port {
            80 | 8080 | 8443 => {
                let methods = ["GET", "POST", "HEAD", "OPTIONS"];
                let method = methods[self.rng.gen_range(0..methods.len())];
                let statuses = [200, 301, 404, 500];
                let status = statuses[self.rng.gen_range(0..statuses.len())];
                (
                    "HTTP/1.1",
                    DetectionMethod::ActiveProbe,
                    format!(
                        "{} / HTTP/1.1\r\nHost: example.com\r\n\r\nHTTP/1.1 {} OK\r\n\r\n",
                        method, status
                    ),
                    method.to_string(),
                    status,
                    String::new(),
                )
            }
            443 => (
                "HTTPS",
                DetectionMethod::ActiveProbe,
                "TLS 1.3 Handshake Complete\nServer: nginx/1.25.3".into(),
                String::new(),
                0,
                String::new(),
            ),
            22 => (
                "SSH",
                DetectionMethod::LzrHeuristic,
                "SSH-2.0-OpenSSH_9.6p1 Ubuntu-3ubuntu13.6\r\n".into(),
                String::new(),
                0,
                "OpenSSH_9.6p1".into(),
            ),
            3306 => (
                "MySQL",
                DetectionMethod::ActiveProbe,
                "5.7.42-0ubuntu0.18.04.1-log".into(),
                String::new(),
                0,
                String::new(),
            ),
            5432 => (
                "PostgreSQL",
                DetectionMethod::ActiveProbe,
                "PostgreSQL 16.2 on x86_64-pc-linux-gnu".into(),
                String::new(),
                0,
                String::new(),
            ),
            _ => (
                "Unknown",
                DetectionMethod::PortNumber,
                format!("Banner data for port {}", port),
                String::new(),
                0,
                String::new(),
            ),
        };

        let sanitized = banner.replace('\0', ".");
        let hash = sha2::Sha256::digest(sanitized.as_bytes());
        let banner_hash = hex::encode(hash);

        L7Banner {
            event_id: Uuid::new_v4().to_string(),
            target_ip: ip,
            target_port: port,
            transport: Transport::Tcp as i32,
            protocol: protocol.into(),
            detection_method: detection as i32,
            raw_banner: banner.into_bytes(),
            sanitized_banner: sanitized,
            banner_hash,
            http_method,
            http_status_code: http_status,
            http_server: if protocol == "HTTP/1.1" {
                "nginx/1.25.3".into()
            } else {
                String::new()
            },
            ssh_version,
            ssh_key_exchange_algorithms: if port == 22 {
                "curve25519-sha256".into()
            } else {
                String::new()
            },
            ssh_host_key_type: if port == 22 {
                "ssh-ed25519".into()
            } else {
                String::new()
            },
            db_version: String::new(),
            source_pop: self.source_pop.clone(),
            observed_at: now.to_rfc3339(),
        }
    }

    fn random_ip_bytes(&mut self) -> Vec<u8> {
        vec![
            self.rng.gen_range(1..224),
            self.rng.gen_range(0..255),
            self.rng.gen_range(0..255),
            self.rng.gen_range(1..254),
        ]
    }

    fn random_l4_result(&mut self) -> i32 {
        let r: u8 = self.rng.gen_range(0..100);
        if r < 70 {
            L4Result::Open as i32
        } else if r < 90 {
            L4Result::Closed as i32
        } else {
            L4Result::Filtered as i32
        }
    }

    fn random_port(&mut self) -> u32 {
        let ports = [
            80, 443, 22, 21, 25, 53, 110, 143, 993, 995, 3306, 5432, 6379, 27017, 8080, 8443, 8888,
            9090, 3000, 5000,
        ];
        ports[self.rng.gen_range(0..ports.len())]
    }

    fn random_tls_port(&mut self) -> u32 {
        let ports = [443, 8443, 993, 995, 465, 636, 8883];
        ports[self.rng.gen_range(0..ports.len())]
    }

    fn random_domain(&mut self) -> String {
        let domains = [
            "example.com",
            "test.org",
            "demo.net",
            "sample.io",
            "cloud.dev",
        ];
        domains[self.rng.gen_range(0..domains.len())].to_string()
    }

    fn random_org(&mut self) -> String {
        let orgs = [
            "Amazon Web Services",
            "Google Cloud",
            "Microsoft Azure",
            "Cloudflare",
            "DigitalOcean",
            "Hetzner",
            "OVH",
            "Linode",
            "Vultr",
            "Oracle Cloud",
        ];
        orgs[self.rng.gen_range(0..orgs.len())].to_string()
    }

    fn random_hex(&mut self, len: usize) -> String {
        (0..len)
            .map(|_| {
                let idx = self.rng.gen_range(0..16);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'a' + idx - 10) as char
                }
            })
            .collect()
    }
}
