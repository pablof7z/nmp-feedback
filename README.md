# nmp-feedback

Reusable NMP feedback module.

`nmp-feedback` owns the feedback protocol behavior that should not be repeated
in app shells:

- project-scoped kind:1 and kind:513 interests
- explicit relay-targeted feedback publishing through `nmp.publish`
- NIP-70 protected feedback tags
- bounded event caching from NMP `KernelEventObserver`
- thread projection from flat events into roots, replies, and newest metadata

Apps keep their own project coordinate and native presentation. The app shell
passes semantic actions like `FetchFeedback` and `PublishFeedback`; this module
builds the Nostr/NMP details and returns snapshot-ready DTOs.

## Intended Use

```rust
let config = nmp_feedback::FeedbackConfig::new(PROJECT_COORDINATE)
    .with_interest_namespace("myapp.feedback");

let runtime = nmp_feedback::FeedbackRuntime::new(config, Arc::new(Mutex::new(Vec::new())), rev)
    .with_snapshot_bump(Arc::new(move || snapshot_signal.bump()));

app.register_event_observer(Arc::new(runtime.observer()));

let events = runtime.snapshot_events();
let threads = runtime.snapshot_threads();
```

Native iOS and Android shells should render `snapshot_threads()` and dispatch
`runtime.fetch(...)` / `runtime.publish(...)` through their NMP app action
surface. The native shell may own shake detection and composition UI, but not
relay sockets, event tags, signing policy, or thread reduction.
