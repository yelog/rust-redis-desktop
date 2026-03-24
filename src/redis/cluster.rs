use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    pub id: String,
    pub endpoint: String,
    pub flags: String,
    pub master_id: String,
    pub ping_sent: u64,
    pub pong_recv: u64,
    pub config_epoch: u64,
    pub link_state: String,
    pub slots: Vec<SlotRange>,
    pub is_master: bool,
    pub is_replica: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotRange {
    pub start: u16,
    pub end: u16,
}

impl SlotRange {
    pub fn contains(&self, slot: u16) -> bool {
        slot >= self.start && slot <= self.end
    }

    pub fn count(&self) -> u16 {
        self.end - self.start + 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub state: String,
    pub slots_assigned: u64,
    pub slots_ok: u64,
    pub slots_pfail: u64,
    pub slots_fail: u64,
    pub known_nodes: u64,
    pub size: u64,
    pub current_epoch: u64,
    pub my_epoch: u64,
    pub stats_messages_sent: u64,
    pub stats_messages_received: u64,
}

impl Default for ClusterInfo {
    fn default() -> Self {
        Self {
            state: "unknown".to_string(),
            slots_assigned: 0,
            slots_ok: 0,
            slots_pfail: 0,
            slots_fail: 0,
            known_nodes: 0,
            size: 0,
            current_epoch: 0,
            my_epoch: 0,
            stats_messages_sent: 0,
            stats_messages_received: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SentinelInfo {
    pub master_name: String,
    pub master_host: String,
    pub master_port: u16,
    pub master_link_status: String,
    pub master_host_down: bool,
    pub num_slaves: u64,
    pub num_sentinels: u64,
    pub quorum: u64,
    pub failover_timeout: u64,
    pub parallel_syncs: u64,
    pub slaves: Vec<ReplicaInfo>,
    pub sentinels: Vec<SentinelNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub state: String,
    pub master_link_status: String,
    pub master_host: String,
    pub master_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelNode {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub runid: String,
    pub flags: String,
    pub last_ping: i64,
    pub last_ok_ping: i64,
    pub last_ping_sent: i64,
    pub master_host: String,
    pub master_port: u16,
}

pub fn parse_cluster_nodes(output: &str) -> Vec<ClusterNode> {
    let mut nodes = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 8 {
            continue;
        }

        let id = parts[0].to_string();
        let endpoint_parts: Vec<&str> = parts[1].split('@').collect();
        let endpoint = endpoint_parts[0].to_string();
        let flags = parts[2].to_string();
        let master_id = parts[3].to_string();

        let is_master = flags.contains("master");
        let is_replica = flags.contains("slave") || flags.contains("replica");

        let mut slots = Vec::new();
        for slot_part in parts.iter().skip(8) {
            if let Some(range) = parse_slot_range(slot_part) {
                slots.push(range);
            }
        }

        let ping_sent = parts[4].parse().unwrap_or(0);
        let pong_recv = parts[5].parse().unwrap_or(0);
        let config_epoch = parts[6].parse().unwrap_or(0);
        let link_state = parts[7].to_string();

        nodes.push(ClusterNode {
            id,
            endpoint,
            flags,
            master_id,
            ping_sent,
            pong_recv,
            config_epoch,
            link_state,
            slots,
            is_master,
            is_replica,
        });
    }

    nodes
}

fn parse_slot_range(s: &str) -> Option<SlotRange> {
    let s = s.trim_start_matches('[').trim_end_matches(']');

    if s.contains('-') {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 2 {
            let start = parts[0].parse().ok()?;
            let end = parts[1].parse().ok()?;
            return Some(SlotRange { start, end });
        }
    } else if let Ok(slot) = s.parse::<u16>() {
        return Some(SlotRange {
            start: slot,
            end: slot,
        });
    }

    None
}

pub fn parse_cluster_info(output: &str) -> ClusterInfo {
    let mut info = ClusterInfo::default();

    for line in output.lines() {
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "cluster_state" => info.state = value.trim().to_string(),
                "cluster_slots_assigned" => info.slots_assigned = value.trim().parse().unwrap_or(0),
                "cluster_slots_ok" => info.slots_ok = value.trim().parse().unwrap_or(0),
                "cluster_slots_pfail" => info.slots_pfail = value.trim().parse().unwrap_or(0),
                "cluster_slots_fail" => info.slots_fail = value.trim().parse().unwrap_or(0),
                "cluster_known_nodes" => info.known_nodes = value.trim().parse().unwrap_or(0),
                "cluster_size" => info.size = value.trim().parse().unwrap_or(0),
                "cluster_current_epoch" => info.current_epoch = value.trim().parse().unwrap_or(0),
                "cluster_my_epoch" => info.my_epoch = value.trim().parse().unwrap_or(0),
                "cluster_stats_messages_sent" => {
                    info.stats_messages_sent = value.trim().parse().unwrap_or(0)
                }
                "cluster_stats_messages_received" => {
                    info.stats_messages_received = value.trim().parse().unwrap_or(0)
                }
                _ => {}
            }
        }
    }

    info
}

pub fn parse_sentinel_masters(output: &str) -> Vec<SentinelInfo> {
    let mut masters = Vec::new();

    for block in output.split("\n\n") {
        if block.trim().is_empty() {
            continue;
        }

        let mut info = SentinelInfo::default();
        let mut fields = HashMap::new();

        for line in block.lines() {
            if let Some((key, value)) = line.split_once(':') {
                fields.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        if let Some(name) = fields.get("name") {
            info.master_name = name.clone();
        }
        if let Some(host) = fields.get("ip") {
            info.master_host = host.clone();
        }
        if let Some(port) = fields.get("port") {
            info.master_port = port.parse().unwrap_or(6379);
        }
        if let Some(status) = fields.get("master_link_status") {
            info.master_link_status = status.clone();
            info.master_host_down = status != "ok";
        }
        if let Some(num) = fields.get("num-slaves") {
            info.num_slaves = num.parse().unwrap_or(0);
        }
        if let Some(num) = fields.get("num-other-sentinels") {
            info.num_sentinels = num.parse().unwrap_or(0);
        }
        if let Some(quorum) = fields.get("quorum") {
            info.quorum = quorum.parse().unwrap_or(0);
        }

        masters.push(info);
    }

    masters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cluster_info() {
        let output = r#"cluster_state:ok
cluster_slots_assigned:16384
cluster_slots_ok:16384
cluster_slots_pfail:0
cluster_slots_fail:0
cluster_known_nodes:6
cluster_size:3"#;

        let info = parse_cluster_info(output);
        assert_eq!(info.state, "ok");
        assert_eq!(info.slots_assigned, 16384);
        assert_eq!(info.known_nodes, 6);
        assert_eq!(info.size, 3);
    }

    #[test]
    fn test_slot_range_contains() {
        let range = SlotRange {
            start: 0,
            end: 5460,
        };
        assert!(range.contains(100));
        assert!(range.contains(5460));
        assert!(!range.contains(5461));
    }

    #[test]
    fn test_slot_range_count() {
        let range = SlotRange {
            start: 0,
            end: 5460,
        };
        assert_eq!(range.count(), 5461);
    }
}
