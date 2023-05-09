# **0 - Hello, Clockwork**

For a complete guide to this example project, please see to the [Clockwork docs](https://docs.clockwork.xyz/developers/guides/0-hello-clockwork).

---

Testing locally:
```bash
cargo install -f --locked clockwork-cli
clockwork localnet
```

Get a new program id:
```bash
./new-program-id.sh <localnet|devnet>
```

Run the tests and observe the logs:
```bash
anchor test
```
