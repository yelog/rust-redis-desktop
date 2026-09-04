use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandStat {
    pub command: String,
    pub calls: u64,
    pub usec: u64,
    pub usec_per_call: f64,
    pub rejected_calls: u64,
    pub failed_calls: u64,
}

pub fn parse_command_stats(info: &str) -> Vec<CommandStat> {
    let mut stats = info
        .lines()
        .filter_map(|line| {
            let (key, value) = line.trim().split_once(':')?;
            let command = key.strip_prefix("cmdstat_")?.to_string();
            let mut fields = HashMap::new();
            for field in value.split(',') {
                let (name, value) = field.split_once('=')?;
                fields.insert(name.trim(), value.trim());
            }
            Some(CommandStat {
                command,
                calls: fields.get("calls")?.parse().ok()?,
                usec: fields.get("usec").and_then(|v| v.parse().ok()).unwrap_or(0),
                usec_per_call: fields
                    .get("usec_per_call")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.0),
                rejected_calls: fields
                    .get("rejected_calls")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                failed_calls: fields
                    .get("failed_calls")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
            })
        })
        .collect::<Vec<_>>();
    stats.sort_by(|left, right| right.calls.cmp(&left.calls));
    stats
}

pub fn command_stat_rates(
    previous: &[CommandStat],
    current: &[CommandStat],
    elapsed_secs: f64,
) -> Vec<(String, f64)> {
    if elapsed_secs <= 0.0 {
        return Vec::new();
    }
    let previous_by_name: HashMap<&str, u64> = previous
        .iter()
        .map(|stat| (stat.command.as_str(), stat.calls))
        .collect();
    let mut rates = current
        .iter()
        .filter_map(|stat| {
            let previous_calls = previous_by_name
                .get(stat.command.as_str())
                .copied()
                .unwrap_or(0);
            (stat.calls >= previous_calls).then(|| {
                (
                    stat.command.clone(),
                    (stat.calls - previous_calls) as f64 / elapsed_secs,
                )
            })
        })
        .collect::<Vec<_>>();
    rates.sort_by(|left, right| right.1.total_cmp(&left.1));
    rates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_sorts_command_stats() {
        let stats = parse_command_stats(
            "# Commandstats\ncmdstat_get:calls=20,usec=100,usec_per_call=5.0,rejected_calls=0,failed_calls=1\ncmdstat_set:calls=30,usec=90,usec_per_call=3.0,rejected_calls=2,failed_calls=0\n",
        );
        assert_eq!(stats[0].command, "set");
        assert_eq!(stats[0].rejected_calls, 2);
        assert_eq!(stats[1].failed_calls, 1);
    }

    #[test]
    fn calculates_rates_and_ignores_counter_resets() {
        let previous = vec![CommandStat {
            command: "get".to_string(),
            calls: 10,
            ..Default::default()
        }];
        let current = vec![
            CommandStat {
                command: "get".to_string(),
                calls: 30,
                ..Default::default()
            },
            CommandStat {
                command: "set".to_string(),
                calls: 2,
                ..Default::default()
            },
        ];
        assert_eq!(
            command_stat_rates(&previous, &current, 2.0),
            vec![("get".to_string(), 10.0), ("set".to_string(), 1.0)]
        );
    }
}
