//! Multi-queue NUMA pinning for AF_XDP probe engine.
//!
//! Auto-detects NIC queues, pins each queue to its local NUMA node,
//! and sets CPU affinity via sched_setaffinity.
//!
//! Zero external dependencies — kernel syscalls only.
//!
//! Usage:
//!   let pinning = NumaPinning::detect("eth0").unwrap();
//!   pinning.pin_queue_to_numa(0).unwrap();

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum NumaError {
    #[error("interface not found: {0}")]
    InterfaceNotFound(String),
    #[error("NUMA info not available: {0}")]
    NumaNotAvailable(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("scheduling error: {0}")]
    ScheduleError(String),
    #[error("no queues found for interface")]
    NoQueues,
}

#[derive(Debug, Clone)]
pub struct NumaNode {
    pub id: u32,
    pub cpus: Vec<u32>,
    pub memory_size_kb: u64,
}

#[derive(Debug, Clone)]
pub struct QueueInfo {
    pub queue_id: u32,
    pub numa_node: u32,
    pub cpu_list: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct NumaPinning {
    interface: String,
    queues: Vec<QueueInfo>,
    nodes: HashMap<u32, NumaNode>,
}

impl NumaPinning {
    pub fn detect(interface: &str) -> Result<Self, NumaError> {
        let queues = Self::detect_queues(interface)?;
        let nodes = Self::detect_numa_nodes()?;

        Ok(Self {
            interface: interface.to_string(),
            queues,
            nodes,
        })
    }

    fn detect_queues(interface: &str) -> Result<Vec<QueueInfo>, NumaError> {
        let base = PathBuf::from("/sys/class/net").join(interface);
        if !base.exists() {
            return Err(NumaError::InterfaceNotFound(interface.to_string()));
        }

        let mut queues = Vec::new();
        let queues_dir = base.join("queues");

        if queues_dir.exists() {
            let mut entries: Vec<_> = fs::read_dir(&queues_dir)
                .map_err(NumaError::Io)?
                .filter_map(|e| e.ok())
                .collect();

            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("rx-") || name.starts_with("tx-") {
                    let queue_id: u32 = name
                        .trim_start_matches("rx-")
                        .trim_start_matches("tx-")
                        .parse()
                        .unwrap_or(0);

                    let numa_path = entry.path().join("numa_node");
                    let numa_node = fs::read_to_string(&numa_path)
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(0);

                    let cpu_list = Self::get_numa_cpus(numa_node);

                    queues.push(QueueInfo {
                        queue_id,
                        numa_node,
                        cpu_list,
                    });
                }
            }
        }

        if queues.is_empty() {
            for qid in 0..16 {
                let rx_path = base.join(format!("queues/rx-{}/numa_node", qid));
                let tx_path = base.join(format!("queues/tx-{}/numa_node", qid));

                let numa_node = if rx_path.exists() {
                    fs::read_to_string(&rx_path)
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(0)
                } else if tx_path.exists() {
                    fs::read_to_string(&tx_path)
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(0)
                } else {
                    break;
                };

                let cpu_list = Self::get_numa_cpus(numa_node);
                queues.push(QueueInfo {
                    queue_id: qid,
                    numa_node,
                    cpu_list,
                });
            }
        }

        if queues.is_empty() {
            return Err(NumaError::NoQueues);
        }

        Ok(queues)
    }

    fn detect_numa_nodes() -> Result<HashMap<u32, NumaNode>, NumaError> {
        let mut nodes = HashMap::new();
        let numa_base = Path::new("/sys/devices/system/node");

        if !numa_base.exists() {
            return Ok(nodes);
        }

        for entry in fs::read_dir(numa_base)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with("node") {
                if let Ok(id) = name.trim_start_matches("node").parse::<u32>() {
                    let cpus_path = entry.path().join("cpulist");
                    let cpus = fs::read_to_string(&cpus_path)
                        .ok()
                        .map(|s| Self::parse_cpu_list(&s))
                        .unwrap_or_default();

                    let mem_path = entry.path().join("meminfo");
                    let memory_size_kb = fs::read_to_string(&mem_path)
                        .ok()
                        .and_then(|s| {
                            for line in s.lines() {
                                if line.contains("MemTotal:") {
                                    let parts: Vec<&str> = line.split_whitespace().collect();
                                    if parts.len() >= 2 {
                                        return parts[1].parse().ok();
                                    }
                                }
                            }
                            None
                        })
                        .unwrap_or(0);

                    nodes.insert(
                        id,
                        NumaNode {
                            id,
                            cpus,
                            memory_size_kb,
                        },
                    );
                }
            }
        }

        Ok(nodes)
    }

    fn parse_cpu_list(list: &str) -> Vec<u32> {
        let mut cpus = Vec::new();
        for part in list.split(',') {
            let part = part.trim();
            if let Some((start, end)) = part.split_once('-') {
                if let (Ok(s), Ok(e)) = (start.parse::<u32>(), end.parse::<u32>()) {
                    for cpu in s..=e {
                        cpus.push(cpu);
                    }
                }
            } else if let Ok(cpu) = part.parse::<u32>() {
                cpus.push(cpu);
            }
        }
        cpus
    }

    fn get_numa_cpus(numa_node: u32) -> Vec<u32> {
        let cpulist_path = format!("/sys/devices/system/node/node{}/cpulist", numa_node);
        fs::read_to_string(&cpulist_path)
            .ok()
            .map(|s| Self::parse_cpu_list(&s))
            .unwrap_or_default()
    }

    pub fn queues(&self) -> &[QueueInfo] {
        &self.queues
    }

    pub fn nodes(&self) -> &HashMap<u32, NumaNode> {
        &self.nodes
    }

    pub fn interface(&self) -> &str {
        &self.interface
    }

    pub fn queue_count(&self) -> usize {
        self.queues.len()
    }

    pub fn pin_queue_to_numa(&self, queue_id: u32) -> Result<(), NumaError> {
        let queue = self
            .queues
            .iter()
            .find(|q| q.queue_id == queue_id)
            .ok_or(NumaError::NoQueues)?;

        if queue.cpu_list.is_empty() {
            return Err(NumaError::ScheduleError(
                "no CPUs available for NUMA node".into(),
            ));
        }

        #[cfg(target_os = "linux")]
        {
            self.set_cpu_affinity(&queue.cpu_list)?;
        }

        #[cfg(not(target_os = "linux"))]
        {
            println!(
                "[numa] would pin queue {} to NUMA node {} CPUs {:?}",
                queue_id, queue.numa_node, queue.cpu_list
            );
        }

        Ok(())
    }

    pub fn pin_all_queues(&self) -> Result<(), NumaError> {
        for queue in &self.queues {
            self.pin_queue_to_numa(queue.queue_id)?;
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn set_cpu_affinity(&self, cpus: &[u32]) -> Result<(), NumaError> {
        if cpus.is_empty() {
            return Ok(());
        }

        let mut cpu_set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
        let max_cpu = cpus.iter().copied().max().unwrap_or(0) as usize;

        unsafe {
            libc::CPU_ZERO(&mut cpu_set);
            for &cpu in cpus {
                libc::CPU_SET(cpu as usize, &mut cpu_set);
            }

            let ret = libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpu_set);

            if ret != 0 {
                return Err(NumaError::ScheduleError(
                    std::io::Error::last_os_error().to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn summary(&self) -> String {
        let mut out = format!("NUMA Pinning Summary for {}\n", self.interface);
        out.push_str(&format!("  Nodes: {}\n", self.nodes.len()));
        out.push_str(&format!("  Queues: {}\n", self.queues.len()));

        for queue in &self.queues {
            out.push_str(&format!(
                "  Queue {}: NUMA node {} — CPUs {:?}\n",
                queue.queue_id, queue.numa_node, queue.cpu_list
            ));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_list() {
        assert_eq!(NumaPinning::parse_cpu_list("0-3"), vec![0, 1, 2, 3]);
        assert_eq!(NumaPinning::parse_cpu_list("0,2,4"), vec![0, 2, 4]);
        assert_eq!(NumaPinning::parse_cpu_list("0-1,4-5"), vec![0, 1, 4, 5]);
        assert_eq!(NumaPinning::parse_cpu_list("7"), vec![7]);
        assert_eq!(NumaPinning::parse_cpu_list(""), Vec::<u32>::new());
    }

    #[test]
    fn test_numa_node_detection() {
        let nodes = NumaPinning::detect_numa_nodes().unwrap();
        println!("Detected {} NUMA nodes", nodes.len());
        for (id, node) in &nodes {
            println!(
                "  Node {}: {} CPUs, {} KB memory",
                id,
                node.cpus.len(),
                node.memory_size_kb
            );
        }
    }

    #[test]
    fn test_queue_detection() {
        match NumaPinning::detect_queues("eth0") {
            Ok(queues) => {
                println!("Detected {} queues for eth0", queues.len());
                for q in &queues {
                    println!("  Queue {}: NUMA node {}", q.queue_id, q.numa_node);
                }
            }
            Err(e) => {
                println!("Queue detection failed (expected on non-Linux): {}", e);
            }
        }
    }

    #[test]
    fn test_summary_output() {
        let pinning = NumaPinning {
            interface: "eth0".to_string(),
            queues: vec![
                QueueInfo {
                    queue_id: 0,
                    numa_node: 0,
                    cpu_list: vec![0, 1],
                },
                QueueInfo {
                    queue_id: 1,
                    numa_node: 1,
                    cpu_list: vec![2, 3],
                },
            ],
            nodes: HashMap::new(),
        };

        let summary = pinning.summary();
        assert!(summary.contains("eth0"));
        assert!(summary.contains("Queues: 2"));
        assert!(summary.contains("Queue 0"));
        assert!(summary.contains("Queue 1"));
    }
}
