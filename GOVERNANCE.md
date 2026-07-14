# Hotaru Governance and Component Ownership

This document explains where technical questions, issues, and pull requests
should be routed. Hotaru uses two levels of ownership:

1. A **family** groups components with the same architectural role.
2. A **component** maps that role to concrete files and has a primary
   maintainer.

The ownership hierarchy is:

```text
Family maintainer(s)
└── Component maintainer(s)
    └── Contributors
```

## Family ownership

| Family | Scope | Current family maintainer |
| --- | --- | --- |
| Core framework | Core contracts, DSL, and routing | [@Redstone-D](https://github.com/Redstone-D) |
| Facade and tooling | Public facade, feature wiring, CLI, project templates, and shared utilities | [@JerrySu5379](https://github.com/JerrySu5379) |
| Protocol implementations | HTTP/TLS, web middleware, MQTT, and experimental protocols | [@Redstone-D](https://github.com/Redstone-D) |
| Runtime implementations | Tokio and Embassy runtime adapters | [@JerrySu5379](https://github.com/JerrySu5379) |
| I/O implementations | Tokio, Futures, and embedded I/O adapters | [@JerrySu5379](https://github.com/JerrySu5379) |
| Project-wide coordination | Workspace configuration, CI, releases, governance, and cross-component examples | [@Redstone-D](https://github.com/Redstone-D) |

Family maintainers oversee every component in their family. The component
tables below identify the delegated technical contact for each path.

## How ownership works

Technical maintainership is separate from community moderation under the
[Code of Conduct](./CODE_OF_CONDUCT.md). A component maintainer is the first
technical contact, not the sole person allowed to contribute or make design
suggestions.

The family maintainer:

- is accountable for the family-level architecture and every component within
  the family;
- delegates day-to-day technical ownership to component maintainers and
  coordinates their work;
- assigns new or unclear work to the appropriate component;
- coordinates reviews for changes spanning multiple components; and
- has final responsibility for family-level decisions after consulting the
  affected component maintainers.

The primary component maintainer:

- maintains the component under authority delegated by the family maintainer;
- answers or forwards questions about the component;
- provides design context and reviews changes;
- keeps the component's tests and documentation aligned with its behavior; and
- requests cross-component review when a change affects shared contracts.

The backup maintainer handles component questions when the primary maintainer
is unavailable. Questions that cannot be resolved at the component level are
escalated to the current family maintainer or maintainers.

## AI declarations

The AI-assistance tiers are defined in the
[README](./readme.md#ai-assisted-development). Each component's declaration is
listed below. **Not yet declared** is a tracking status, not a tier.

## Component ownership

### Core framework

| Component | Files and directories | Primary | Backup | AI declaration |
| --- | --- | --- | --- | --- |
| Core contracts and semantics | `hotaru_core/**`, except the routing paths listed below | [@Redstone-D](https://github.com/Redstone-D) | [@JerrySu5379](https://github.com/JerrySu5379) | **Author-Owned** for `app`, `connection`, `executable`, and `protocol`; remaining paths not yet declared |
| DSL and procedural macros | `hotaru_trans/**` | [@Redstone-D](https://github.com/Redstone-D) | [@JerrySu5379](https://github.com/JerrySu5379) | **Author-Owned** for `endpoint`, `outpoint`, and `middleware`, whose proof and language design must be human-understood; remaining paths not yet declared |

### Facade and tooling

| Component | Files and directories | Primary | Backup | AI declaration |
| --- | --- | --- | --- | --- |
| Routing and URL semantics | `hotaru_core/src/url.rs`, `hotaru_core/src/url/**` | [@JerrySu5379](https://github.com/JerrySu5379) | [@Redstone-D](https://github.com/Redstone-D) | **Author-Owned** |
| Facade and public feature surface | `hotaru/src/lib.rs`, `hotaru/src/prelude.rs`, `hotaru/src/http.rs`, `hotaru/src/test.rs`, `hotaru/Cargo.toml`, `hotaru/readme.md` | [@Redstone-D](https://github.com/Redstone-D) | [@JerrySu5379](https://github.com/JerrySu5379) | Not yet declared |
| CLI and project templates | `hotaru/src/main.rs`, `templates/**`, `programfiles/**`, `hotaru_style_guide/**` | [@Redstone-D](https://github.com/Redstone-D) | [@JerrySu5379](https://github.com/JerrySu5379) | Not yet declared |
| Shared utilities | `hotaru_lib/**` | [@Redstone-D](https://github.com/Redstone-D) | [@JerrySu5379](https://github.com/JerrySu5379) | **Human-Led**; basic API access |

### Protocol implementations

| Component | Files and directories | Primary | Backup | AI declaration |
| --- | --- | --- | --- | --- |
| HTTP, TLS, and standard web middleware | `hotaru_http/**`, `hotaru_tls/**`, `htmstd/**`, `ahttpm/**` | [@Redstone-D](https://github.com/Redstone-D) | [@JerrySu5379](https://github.com/JerrySu5379) | **Human-Led** for `hotaru_http/**`, `htmstd/cors/**`, and `htmstd/session/**`; **Co-Authored** for the Akari integration in `ahttpm/**`; `hotaru_tls/**` and other middleware not yet declared |
| MQTT client and broker | [`Field-of-Dream-Studio/hotaru_mqtt`](https://github.com/Field-of-Dream-Studio/hotaru_mqtt) | [@JerrySu5379](https://github.com/JerrySu5379) | [@Redstone-D](https://github.com/Redstone-D) | **Human-Led**, with broker and traits **Co-Authored** |
| Experimental protocol integrations | `h2per/**`, `hotaru_grpc/**` | [@Redstone-D](https://github.com/Redstone-D) and [@JerrySu5379](https://github.com/JerrySu5379) | — | Unstable Hyper integration in `h2per/**` **Co-Authored**; `hotaru_grpc/**` not yet declared |

The MQTT repository should keep its own matching ownership file. This row
records where Hotaru contributors should initially route MQTT questions.

### Runtime implementations

| Component | Files and directories | Primary | Backup | AI declaration |
| --- | --- | --- | --- | --- |
| Tokio runtime | `hotaru_rt_tokio/**` | [@JerrySu5379](https://github.com/JerrySu5379) | [@Redstone-D](https://github.com/Redstone-D) | Not yet declared |
| Embassy runtime | `hotaru_rt_embassy/**` | [@zkmaojack](https://github.com/zkmaojack) | [@JerrySu5379](https://github.com/JerrySu5379) | Not yet declared |

### I/O implementations

| Component | Files and directories | Primary | Backup | AI declaration |
| --- | --- | --- | --- | --- |
| Tokio I/O | `hotaru_io_tokio/**` | [@JerrySu5379](https://github.com/JerrySu5379) | [@Redstone-D](https://github.com/Redstone-D) | Not yet declared |
| Futures I/O | `hotaru_io_futures/**` | [@JerrySu5379](https://github.com/JerrySu5379) | [@Redstone-D](https://github.com/Redstone-D) | Not yet declared |
| Embedded I/O | `hotaru_io_embedded/**` | [@zkmaojack](https://github.com/zkmaojack) | [@JerrySu5379](https://github.com/JerrySu5379) | Not yet declared |

### Project-wide coordination

| Component | Files and directories | Primary reviewers | AI declaration |
| --- | --- | --- | --- |
| Workspace, CI, releases, and governance | root-level workspace files and documentation, `.github/**` | [@Redstone-D](https://github.com/Redstone-D) and [@JerrySu5379](https://github.com/JerrySu5379) | Not yet declared |
| Examples | `examples/**` | Maintainer of the component demonstrated by the example | Inherits the declaration of the demonstrated component |

An example that combines multiple components should be reviewed by the
maintainers of each affected component.

## Cross-cutting review

Some changes belong to one component by path but affect several components by
behavior. In those cases, use the path owner as the first contact and also
request the following review:

| Change | Additional review |
| --- | --- |
| Shared `RuntimeSpec` or runtime behavior | Maintainers of every affected runtime implementation |
| Shared transport, stream, or I/O contract | Maintainers of every affected I/O implementation |
| Tokio feature behavior or integration | [@JerrySu5379](https://github.com/JerrySu5379) and the affected component maintainer |
| `no_std` or embedded compatibility | [@JerrySu5379](https://github.com/JerrySu5379), plus [@zkmaojack](https://github.com/zkmaojack) when Embassy runtime or embedded I/O is affected |
| Protocol or routing contract | Routing maintainer and maintainers of affected protocol implementations |
| Public facade API or Cargo feature wiring | Facade maintainer and maintainers of affected implementations |
| HTTP/TLS security behavior | HTTP/TLS maintainer |

For example, a change entirely inside `hotaru_rt_embassy/**` is routed to Jack.
A change to a runtime trait in `hotaru_core/**` is routed first to the core
maintainer and also requires review from Jack and any other affected runtime
maintainers.

## Routing a question or change

1. Find the most specific path in the ownership map.
2. Contact the primary component maintainer.
3. Add the relevant cross-cutting reviewers if the behavior crosses component
   boundaries.
4. If no path matches, contact the Project-wide coordination family
   maintainer, who will assign the component before the change is merged.

Tests located inside a crate inherit that crate or subcomponent's ownership.
Changes spanning multiple rows require review from each affected primary or an
explicit delegate.
