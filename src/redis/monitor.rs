use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorEvent {
    pub timestamp: String,
    pub database: u8,
    pub command: String,
    pub args: Vec<String>,
}

pub fn parse_monitor_line(line: &str) -> Option<MonitorEvent> {
    let (timestamp, rest) = line.split_once(' ')?;
    let metadata_end = rest.find("] ")?;
    let metadata = &rest[1..metadata_end];
    let database = metadata.split_whitespace().next()?.parse().ok()?;
    let payload = &rest[metadata_end + 2..];
    let tokens = tokenize(payload);
    let command = tokens.first()?.to_lowercase();
    Some(MonitorEvent {
        timestamp: timestamp.to_string(),
        database,
        command,
        args: tokens.into_iter().skip(1).collect(),
    })
}

fn tokenize(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if let Some(expected) = quote {
            if ch == expected {
                quote = None;
            } else {
                current.push(ch);
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HotKeyStat {
    pub key: String,
    pub calls: u64,
    pub reads: u64,
    pub writes: u64,
}

pub fn aggregate_hot_keys(events: &[MonitorEvent]) -> Vec<HotKeyStat> {
    let mut stats: HashMap<String, HotKeyStat> = HashMap::new();
    for event in events {
        let Some(key) = event.args.first() else {
            continue;
        };
        let stat = stats.entry(key.clone()).or_insert_with(|| HotKeyStat {
            key: key.clone(),
            ..Default::default()
        });
        stat.calls += 1;
        if is_write_command(&event.command) {
            stat.writes += 1;
        } else {
            stat.reads += 1;
        }
    }
    let mut result: Vec<_> = stats.into_values().collect();
    result.sort_by(|left, right| right.calls.cmp(&left.calls));
    result
}

fn is_write_command(command: &str) -> bool {
    matches!(
        command,
        "set"
            | "del"
            | "incr"
            | "decr"
            | "hset"
            | "sadd"
            | "srem"
            | "lpush"
            | "rpush"
            | "zadd"
            | "zrem"
            | "expire"
            | "setex"
            | "mset"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_monitor_arguments() {
        let event = parse_monitor_line(
            "1710000000.123456 [2 127.0.0.1:1234] \"SET\" \"user:1\" \"hello world\"",
        )
        .unwrap();
        assert_eq!(event.database, 2);
        assert_eq!(event.command, "set");
        assert_eq!(event.args, vec!["user:1", "hello world"]);
    }

    #[test]
    fn aggregates_read_and_write_hot_keys() {
        let events = [
            parse_monitor_line("1 [0 x] GET user:1").unwrap(),
            parse_monitor_line("2 [0 x] SET user:1 value").unwrap(),
            parse_monitor_line("3 [0 x] GET user:1").unwrap(),
        ];
        assert_eq!(
            aggregate_hot_keys(&events)[0],
            HotKeyStat {
                key: "user:1".to_string(),
                calls: 3,
                reads: 2,
                writes: 1,
            }
        );
    }
}
