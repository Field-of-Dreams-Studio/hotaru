# Hotaru Contributor Code Style

This guide covers conventions for code **inside the Hotaru framework crates** (`hotaru_core`, `hotaru_http`, `hotaru_lib`, `hotaru_trans`, etc.).

For conventions around *using* Hotaru in applications, see [HOTARU_STYLE.md](./HOTARU_STYLE.md). For community-facing contribution flow, see [CONTRIBUTING.md](./CONTRIBUTING.md).

---

## Prefer `core::*` over `std::*`

When a type or trait exists in both `core` and `std`, **always import the `core` version inside the framework crates**.
