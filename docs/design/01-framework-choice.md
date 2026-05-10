# Framework choice

## Options

| Framework                  | Language             | MC version              | Pros                                                    | Cons                                                                    |
| -------------------------- | -------------------- | ----------------------- | ------------------------------------------------------- | ----------------------------------------------------------------------- |
| Paper                      | Kotlin               | Latest stable           | De facto plugin server. Mature ecosystem.               | JVM only.                                                               |
| Paper + Rust via IPC       | Kotlin + Rust        | Latest (via Paper)      | Real Bevy app. Crash isolation.                         | High effort. All calls async. Kotlin shim still required. No prior art. |
| Paper + Rust via FFI       | Kotlin + Rust cdylib | Latest (via Paper)      | In-process, low per-call overhead.                      | Rust crashes the JVM. No prior art.                                     |
| Spigot                     | Java                 | Lags Paper slightly     | Parent of Paper.                                        | Strict subset of Paper.                                                 |
| Sponge                     | Java                 | Lags vanilla            | Cleaner API design.                                     | Tiny ecosystem.                                                         |
| Pumpkin (as WASM plugin)   | Rust                 | Tracks latest, pre-1.0  | Pure Rust. Sandboxed plugins.                           | API at v0.1; sandbox blocks shared state. Pumpkin pre-1.0.              |
| Pumpkin (as native plugin) | Rust                 | Tracks latest, pre-1.0  | Pure Rust with full server access.                      | Pre-1.0 instability.                                                    |
| Valence                    | Rust                 | 1.20.x; 1.21 incomplete | Pure Rust framework, full control.                      | High effort chasing MC versions.                                        |
| FerrumC                    | Rust                 | 1.21.8                  | Active, targets 1.21.8.                                 | Pre-prod. Plugin API not landed.                                        |
| Hyperion                   | Rust                 | 1.20.1 (test server)    | Pure Rust + Bevy. Plugin API exists. High perf ceiling. | Aimed at massive PvP events, not RPG. MC version lag.                   |

I'm considering Rust options, because I like Rust ;) I'd prefer to avoid writing Java / Kotlin as a
matter of personal preference.

I think the serious contenders therefore are:

1. Paper + Rust FFI
   1. High effort, no prior art; could fall back on pure Kotlin if it doesn't pan out
   2. Functionally very complete
2. Pumpkin native plugin
   1. Accepts pre-1.0 instability and churn
   2. Functionally incomplete

## Decision

**Paper + Panama Rust FFI**. I can fall back on pure Kotlin if the FFI approach is too high effort.
