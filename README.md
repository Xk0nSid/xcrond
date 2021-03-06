xcrond
=======

A cron server written in rust.


### Installation

#### From crates.io
```sh
$ cargo install xcrond
```

#### From Sources
```sh
$ git clone https://github.com/xk0nsid/xcrond
$ cd xcrond
$ cargo build --release
$ ./target/release/xcrond
$ # If you wanna see logs
$ RUST_LOG=info ./target/release/xcrond
```

### TODOS
- [x] Implement base data structure
- [x] Implement base operations on data structure
- [x] Add `prev` and `next` exec time to `Job`
- [x] Change `time` type of `Event` from `u32` to actual time type
- [x] Add main cron loop
- [x] Add forking and re-scheduling logic (Scheduling provided by [this](https://github.com/xk0nsid/cron) repo.)
- [ ] Add crond config (this is config for server)
- [ ] Add cron scheduling config support (this is config for defining cron
      schedules) via a `Jobfile`. An example `Jobfile` is provided in this repo.
- [ ] Add individual user's `Jobfile` support
- [ ] Execute jobs based on `user` permission
