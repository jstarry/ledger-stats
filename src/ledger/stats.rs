use super::{Ledger, Node, ValidNode};
use std::cmp::min;
use std::fmt;
use std::fmt::Display;
use std::usize;

#[derive(Debug, PartialEq)]
pub struct Stats {
    avg_dag_depth: f64,
    avg_txs_per_depth: f64,
    avg_refs: f64,

    // added
    pct_valid: f64,
    avg_tx_rate: f64,
}

impl Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "AVG DAG DEPTH: {:.3}", self.avg_dag_depth)?;
        writeln!(f, "AVG TXS PER DEPTH: {:.3}", self.avg_txs_per_depth)?;
        writeln!(f, "AVG REFS: {:.3}", self.avg_refs)?;
        writeln!(f, "PCT VALID: {:.1}%", self.pct_valid)?;
        write!(f, "AVG TX RATE: {:.3}", self.avg_tx_rate)
    }
}

impl Ledger {
    pub fn compute_stats(&self) -> Stats {
        let nodes = &self.0;
        let valid_nodes = self.valid_nodes();
        let total_nodes = valid_nodes.len() + 1; // include origin

        let depths = self.compute_depths();
        let depth_sum: usize = depths.iter().sum();
        let avg_dag_depth = depth_sum as f64 / total_nodes as f64;

        let refs_count = self.count_refs();
        let avg_refs = refs_count as f64 / total_nodes as f64;

        let depth_max: usize = depths.iter().max().map_or(0, |&max| max);
        let avg_txs_per_depth = if depth_max > 0 {
            let mut txs_per_depth = vec![0; depth_max];
            depths.into_iter().for_each(|d| txs_per_depth[d - 1] += 1);
            let txs_depth_sum: usize = txs_per_depth.iter().sum();
            txs_depth_sum as f64 / depth_max as f64
        } else {
            0.0
        };

        let pct_valid = if nodes.len() > 0 {
            100.0 * valid_nodes.len() as f64 / nodes.len() as f64
        } else {
            100.0
        };

        let avg_tx_rate = if valid_nodes.len() > 1 {
            let start = valid_nodes.first().unwrap().timestamp;
            let end = valid_nodes.last().unwrap().timestamp;
            let elapsed = end - start + 1; // measure elapsed time inclusively
            valid_nodes.len() as f64 / elapsed as f64
        } else {
            0.0
        };

        Stats {
            avg_dag_depth,
            avg_txs_per_depth,
            avg_refs,
            pct_valid,
            avg_tx_rate,
        }
    }

    fn valid_nodes(&self) -> Vec<&ValidNode> {
        self.0
            .iter()
            .filter_map(|n| match n {
                Node::Invalid => None,
                Node::Valid(node) => Some(node),
            })
            .collect()
    }

    fn compute_depths(&self) -> Vec<usize> {
        let incr_depth = |depth: &usize| match *depth {
            usize::MAX => usize::MAX,
            _ => depth + 1,
        };

        let mut depths: Vec<usize> = vec![0];
        for node in &self.0 {
            let depth = match node {
                Node::Valid(n) => {
                    let left_path_depth = n.left.and_then(|l| depths.get(l)).map(incr_depth);
                    let right_path_depth = n.right.and_then(|r| depths.get(r)).map(incr_depth);
                    min(
                        left_path_depth.unwrap_or(usize::MAX),
                        right_path_depth.unwrap_or(usize::MAX),
                    )
                }
                Node::Invalid => usize::MAX,
            };
            depths.push(depth);
        }

        // exclude origin and nodes without a path to the origin
        depths
            .into_iter()
            .filter(|d| *d != usize::MAX && *d != 0)
            .collect()
    }

    fn count_refs(&self) -> usize {
        let nodes = &self.0;
        nodes
            .iter()
            .map(|node| match node {
                Node::Valid(n) => {
                    let left_ref = n.left.and_then(|l| nodes.get(l)).map_or(0, |_| 1);
                    let right_ref = n.right.and_then(|r| nodes.get(r)).map_or(0, |_| 1);
                    left_ref + right_ref
                }
                Node::Invalid => 0,
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_compute_stats() {
        let database_txt = "5
        1 1 0
        1 2 0
        2 2 1
        3 3 2
        3 4 3";

        let stats = Ledger::parse(BufReader::new(database_txt.as_bytes())).compute_stats();
        assert_eq!(
            stats,
            Stats {
                avg_dag_depth: 4.0 / 3.0,
                avg_txs_per_depth: 2.5,
                avg_refs: 5.0 / 3.0,
                pct_valid: 100.0,
                avg_tx_rate: 1.25,
            }
        );
    }

    #[test]
    fn test_compute_stats_invalid_nodes() {
        let database_txt = "5
        1 1 1
        1 2 0
        2 2 1
        3 3 2
        3 4 3";

        let stats = Ledger::parse(BufReader::new(database_txt.as_bytes())).compute_stats();
        assert_eq!(
            stats,
            Stats {
                avg_dag_depth: 1.5,
                avg_txs_per_depth: 1.0,
                avg_refs: 1.25,
                pct_valid: 60.0,
                avg_tx_rate: 1.0,
            }
        );
    }

    #[test]
    fn test_compute_stats_high_rate() {
        let database_txt = "5
        1 1 1
        2 1 1
        3 1 1
        4 1 1
        5 1 1";

        let stats = Ledger::parse(BufReader::new(database_txt.as_bytes())).compute_stats();
        assert_eq!(
            stats,
            Stats {
                avg_dag_depth: 5.0 / 6.0,
                avg_txs_per_depth: 5.0,
                avg_refs: 5.0 / 3.0,
                pct_valid: 100.0,
                avg_tx_rate: 5.0,
            }
        );
    }

    #[test]
    fn test_compute_stats_cycle() {
        let database_txt = "3
        1 4 1
        1 2 2
        1 3 3";

        let stats = Ledger::parse(BufReader::new(database_txt.as_bytes())).compute_stats();
        assert_eq!(
            stats,
            Stats {
                avg_dag_depth: 2.0 / 3.0,
                avg_txs_per_depth: 2.0,
                avg_refs: 1.0,
                pct_valid: 100.0 * 2.0 / 3.0,
                avg_tx_rate: 1.0,
            }
        );
    }

    #[test]
    fn test_compute_stats_empty() {
        let stats = Ledger::default().compute_stats();
        assert_eq!(
            stats,
            Stats {
                avg_dag_depth: 0.0,
                avg_txs_per_depth: 0.0,
                avg_refs: 0.0,
                pct_valid: 100.0,
                avg_tx_rate: 0.0,
            }
        );
    }

    #[test]
    fn test_compute_stats_none_valid() {
        let stats = Ledger(vec![Node::Invalid]).compute_stats();
        assert_eq!(
            stats,
            Stats {
                avg_dag_depth: 0.0,
                avg_txs_per_depth: 0.0,
                avg_refs: 0.0,
                pct_valid: 0.0,
                avg_tx_rate: 0.0,
            }
        );
    }
}
