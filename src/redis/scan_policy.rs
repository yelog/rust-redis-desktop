#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDecision {
    Start,
    Confirm { db_size: u64, threshold: u64 },
}

pub fn decide_scan(db_size: u64, threshold: u64) -> ScanDecision {
    if threshold > 0 && db_size >= threshold {
        ScanDecision::Confirm { db_size, threshold }
    } else {
        ScanDecision::Start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_below_threshold() {
        assert_eq!(decide_scan(99, 100), ScanDecision::Start);
    }

    #[test]
    fn confirms_at_threshold() {
        assert_eq!(
            decide_scan(100, 100),
            ScanDecision::Confirm {
                db_size: 100,
                threshold: 100
            }
        );
    }

    #[test]
    fn confirms_above_threshold() {
        assert!(matches!(
            decide_scan(101, 100),
            ScanDecision::Confirm { .. }
        ));
    }

    #[test]
    fn zero_disables_confirmation() {
        assert_eq!(decide_scan(u64::MAX, 0), ScanDecision::Start);
    }
}
