запуск

```cargo run --release -- --config=config.dev.toml```

запуск с отображением лога

```RUST_LOG=rbac=info cargo run --release -- --config=config.dev.toml```

тесты 

```cargo test```

бенчмарк

```rustup run nightly cargo bench```