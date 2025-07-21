# rgb-log

## Todo

- [x] bind crate name automatically in log line
- [x] make tokio optional (sync::Mutex)
  - Make buf.rs its own package so a bunch of tokio features can be disabled in the main crate
- [ ] check at compiletime which submodule names are used through the entire source code (is this possible?) Users would no longer have to specify which modules exist manually
- [ ] Log::new should be a simple method, no args.
