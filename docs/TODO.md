# TODO

* [ ] Adventure Component builder
* [ ] Bundle native DSOs into plugin .jar (no more paths into the `target/` directory)
* [ ] Contain / handle Rust panics?
* [ ] Experiment with dialogs for NPC interaction
* [ ] Find a useful strategy for writing tests
* [ ] idea: Make players spawn near(er) to villages
* [ ] idea: Route Rust plugin log messages to server chat based on `CHAT_LOG` filter?
* [ ] produce bukkit / paper API list so I can track what I need to build bindings for
* [ ] Formalize (and justify) using `#[repr(transparent)] struct TraitInst(JObject)` structs for
      concrete instances of an interface where we don't have a concrete type instead of
      `&dyn Trait`. Ultimately, I think it comes down to the nasty restrictions trait objects have.
* [ ] JNI logging subscriber swallows tracing structured fields
* [ ] tracing perfetto trace target - needs to be toggleable during runtime
* [ ] display village boundaries?
