# ledger-stats

I included a prebuilt binary for osx called `ledgerstats` in the root of the repo for convenience. I also added instructions on how to build a fresh binary if this task is not being verified on a Mac.

### Explanation

- I chose to use Rust because I have been using (and loving) the language for about half a year for various side projects.
- Comments are used sparingly because I feel that code should speak for itself. I tried to take care with my naming so that the execution flow and code structure was self explanatory and intuitive.
- Invalid transactions are handled gracefully to add to the robustness of the database.
- Malformed input is guarded against in the parser and there is a suite of test cases covering the many edge cases. Any unrecoverable error will trigger a panic.
- The origin node is not included the calculation of `PCT VALID` or `AVG TX RATE`
- Algorithmic and memory complexity were important considerations but I opted to not optimize too much if I felt the code readability would suffer as a result.

### Custom Stats

1. `PCT VALID` - *percentage of valid transactions in the database*
    - This stat is interesting because a robust ledger should handle invalid transactions gracefully. This stat could reveal that a bad actor is contributing bad transactions or uncover a bug in the system.
1. `AVG TX RATE` - *rate of transactions over some period of time*
    - Scaling is a hot topic these days in the distributed ledger space and if this were a real world system I would want to know the average throughput.

### Building

```sh
# install rust and cargo
curl https://sh.rustup.rs -sSf | sh

# build binary (note: already included in repo)
cargo build

# run tests
cargo test

# run binary
cargo run <args>
```

### Last Notes

I am a little confused by the term **"bipartite graph"** in the task description. My understanding of the term is that it means a graph can be split into two disjoint sets of nodes where nodes only have edges to nodes in the opposite set. It looks to me that this is not applicable to the task because even the example is not a bipartite graph. Would be great to chat this out further :)
