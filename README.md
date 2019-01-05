xcrond
=======

A cron server written in rust.


### Installation
```sh
$ cargo install xcrond
```

### TODOS
- [x] Implement base data structure
- [x] Implement base operations on data structure
- [x] Add `prev` and `next` exec time to `Job`
- [x] Change `time` type of `Event` from `u32` to actual time type
- [x] Add main cron loop
- [ ] Add crond config (this is config for server)
- [ ] Add cron scheduling config support (this is config for defining cron schedules)
- [ ] Add forking and re-scheduling logic
