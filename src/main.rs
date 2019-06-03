mod ledger;

use ledger::Ledger;
use std::env;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_name = args.get(1).expect("expected file name");
    let f = File::open(file_name).expect("file not found");
    let ledger = Ledger::parse(BufReader::new(f));
    let stats = ledger.compute_stats();
    println!("{}", stats);
}
