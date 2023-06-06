# Microservices
The idea of this project is practicing work with microservices in Rust.

The project will consist in 2 microservices and a client that will connect to it;
- [ ] Auth;
- [ ] HealthCheck;

Project Structure:
```
/[root folder]
    |__ proto
        |__ authentication.proto
    |__ src
        |__ /auth-service
            |__ main.rs
        |__ /client
            |__ main.rs
        |__ /health-check-service
            |__ main.rs
    |__ build.rs
    |__ Cargo.toml
```

The build.rs file is used to compile our ProtoBuffs into Rust code. Cargo will run this buildScript before compiling our source code.
</br>

- RUN
```
cargo run --bin auth
cargo run --bin client
cargo run --bin health-check
```

- BUILD
```
cargo build --release
```

- WATCH
```
cargo watch -x run
```

