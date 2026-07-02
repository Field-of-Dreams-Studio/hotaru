# Hotaru Contributor Code Style

This guide covers conventions for code **inside the Hotaru framework crates** (`hotaru_core`, `hotaru_http`, `hotaru_lib`, `hotaru_trans`, etc.).

For conventions around *using* Hotaru in applications, see [HOTARU_STYLE.md](./HOTARU_STYLE.md). For community-facing contribution flow, see [CONTRIBUTING.md](./CONTRIBUTING.md).

---

## Prefer `core::*` over `std::*`

When a type or trait exists in both `core` and `std`, **always import the `core` version inside the framework crates**.

## Annotate core functions with the `av` version attribute

For core functions, add the [`av`](https://crates.io/crates/av) crate's version
attribute on the top of the item. This records the API's stability, the version
it was introduced in, and any relevant notes so the framework's version surface
stays consistent and machine-readable.

Place `#[av::ver(...)]` directly above the item (function, method, or type):

```rust
#[av::ver(
    unstable,
    since = "0.8.1",
    note = "Resumable URL traversal — surface may change",
    date = "2026-05-25"
)]
pub fn walk_cursor(&self) -> WalkCursor<C, TS> {
    // ...
}
```

Common forms:

- **Unstable API** — the surface may still change:
  ```rust
  #[av::ver(unstable, since = "0.8.1", note = "…", date = "…")]
  ```
- **Deprecated API** — kept for compatibility but discouraged:
  ```rust
  #[av::ver(
      deprecated,
      since = "0.8.0",
      note = "Use `register` after parsing the path. …"
  )]
  ```

Keep the `note` short and actionable, and set `since` to the version where the
item was introduced (or, for `deprecated`, where the deprecation took effect).
