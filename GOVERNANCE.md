# Hotaru Governance and Component Ownership

**Effective since 2026.08.01** 

## 1. Project nature and purpose

Hotaru is an FDS-led open-source project. Anyone may use the project, raise an
issue, propose a design, submit a pull request, or review code. Project-wide
governance remains with FDS; family and component authority may be delegated
to eligible FDS or PMINE members.

This document identifies the technical head for each part of Hotaru, defines
the escalation path for decisions, and makes appointments and succession
predictable. Technical maintainership is separate from community moderation
under the [Code of Conduct](./CODE_OF_CONDUCT.md).

## 2. Roles and ownership

- **Project Maintainer** — governs repository-wide policy, permissions,
  releases, security, licensing, and cross-family decisions. Current:
  [@Redstone-D](https://github.com/Redstone-D).
- **Family Maintainer** — the senior technical head of every component in a
  family.
- **Component Maintainer** — the delegated technical head and first contact for
  one component, one rank below the Family Maintainer.
- **Reviewer or Steward** — assists with review and technical guidance without
  final governance or merge authority.

Family and Component Maintainers are both technical heads for a component. The
Family Maintainer may operate directly, approve or block merges, appoint
Component Maintainers, and publish additional family rules. Component
Maintainers must follow those rules. Family rules must be public and may not
conflict with FDS policy, the license, the Code of Conduct, security rules,
required CI, or project-wide governance.

Ordinary component merges may be delegated to Component Maintainers. No
maintainer may be the sole approver of their own change. A cross-family change
requires approval from every affected family. Questions start with the
Component Maintainer, escalate to the Family Maintainer, and finally to the
Project Maintainer.

Live QA follows the role of the final PR owner:

- a Contributor is questioned by the responsible Component Maintainer;
- a Component Maintainer is questioned by the responsible Family Maintainer;
- a Family Maintainer is questioned by the Project Maintainer; and
- the Project Maintainer is questioned by a different Family Maintainer who
  did not author the change and understands the affected code.

A Family Maintainer conducting QA for the Project Maintainer acts only as the
independent questioner for that change and does not acquire project-wide
authority.

### Pull request integration routes

Every change reaches the canonical repository's default branch through one of
two routes:

| Route | Final PR owner | Records and QA |
| --- | --- | --- |
| Standalone canonical PR | The Contributor or Maintainer who submits the complete change | The PR owner completes the Update Report and answers the live QA; the questioner assigned by the QA order above completes and keeps the QA record. |
| Maintainer-staged integration | An authorized Component or Family Maintainer collects related contributions in a maintainer-controlled branch or fork, then submits a consolidated canonical PR | The integrating Maintainer completes one consolidated Update Report and answers the live QA; the questioner assigned by the integrating Maintainer's role completes and keeps the QA record. |

A merge into a staging branch or fork is not acceptance into Hotaru. The
integrating Maintainer must personally review, understand, explain, modify,
test, and debug every change in the consolidated PR. Original contributors
remain credited and accountable for their authored work; the integrating
Maintainer assumes additional responsibility for its design and integration.
The final PR must link the staged contributions and identify their authors.

The person who completes any required form must understand the code covered by
that form and is responsible for the accuracy and technical judgment recorded
in it. A family may also require records for intermediate contributions. These
routes do not alter the rules against self-approval or the approval required
for cross-family changes.

Printable source files and compiled PDFs for the Update Report and live QA are
kept in [`governance/forms/`](./governance/forms/).

Root workspace files, root documentation, and `.github/**` are governed at the
project level. [@Redstone-D](https://github.com/Redstone-D) and
[@JerrySu5379](https://github.com/JerrySu5379) are the primary reviewers.
Examples inherit the ownership and AI declaration of the components they
demonstrate.

**Release governance.** Release scope, readiness, and timing are decided at
the regular Wednesday and Sunday coordination meetings conducted under the
[FDS Administrator Rules](https://doc.fds.moe/policies/admin/). The Project
Maintainer records and carries out the release decision.

**RFC governance.** Each Family Maintainer defines the RFC process for their
family, subject to these project-wide requirements:

1. A patch-level update that changes only the final version component, such as
   `0.a.b` to `0.a.c`, must not break a stable API.
2. A major or breaking public API change must be proposed to the community and
   discussed at an internal meeting before approval.
3. Family RFC rules may be stricter than these requirements, but not weaker.

### Core framework

Core contracts and the procedural-macro DSL.

**Family Maintainer:** [@Redstone-D](https://github.com/Redstone-D)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| Core contracts and semantics | `hotaru_core/**` except the URL paths below | [@Redstone-D](https://github.com/Redstone-D) |
| DSL and procedural macros | `hotaru_trans/**` | [@Redstone-D](https://github.com/Redstone-D) |

### Facade and tooling

Routing, the public facade and feature surface, CLI tooling, templates, and
shared user-facing utilities.

**Family Maintainer:** [@JerrySu5379](https://github.com/JerrySu5379)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| Routing and URL semantics | `hotaru_core/src/url.rs`, `hotaru_core/src/url/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| Facade and public feature surface | `hotaru/src/lib.rs`, `hotaru/src/prelude.rs`, `hotaru/src/http.rs`, `hotaru/src/test.rs`, `hotaru/Cargo.toml`, `hotaru/readme.md` | [@Redstone-D](https://github.com/Redstone-D) |
| CLI and project templates | `hotaru/src/main.rs`, `templates/**`, `programfiles/**`, `hotaru_style_guide/**` | [@Redstone-D](https://github.com/Redstone-D) |
| Shared utilities | `hotaru_lib/**` | [@Redstone-D](https://github.com/Redstone-D) |

### Protocol implementations

Wire protocols, protocol-specific security, and standard middleware.

**Family Maintainer:** [@Redstone-D](https://github.com/Redstone-D)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| HTTP, TLS, and web middleware | `hotaru_http/**`, `hotaru_tls/**`, `htmstd/**`, `ahttpm/**` | [@Redstone-D](https://github.com/Redstone-D) |
| MQTT client and broker | [`Field-of-Dream-Studio/hotaru_mqtt`](https://github.com/Field-of-Dream-Studio/hotaru_mqtt) | [@JerrySu5379](https://github.com/JerrySu5379) |
| Experimental protocol integrations | `h2per/**`, `hotaru_grpc/**` | [@Redstone-D](https://github.com/Redstone-D), [@JerrySu5379](https://github.com/JerrySu5379) |

The MQTT repository should maintain its own matching ownership rules.

### Runtime implementations

Runtime scheduling, spawning, and runtime-specific integration.

**Family Maintainer:** [@JerrySu5379](https://github.com/JerrySu5379)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| Tokio runtime | `hotaru_rt_tokio/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| Embassy runtime | `hotaru_rt_embassy/**` | [@zkmaojack](https://github.com/zkmaojack) |

### I/O implementations

Adapters between Hotaru's transport contracts and concrete I/O ecosystems.

**Family Maintainer:** [@JerrySu5379](https://github.com/JerrySu5379)

| Component | Files and directories | Component Maintainer |
| --- | --- | --- |
| Tokio I/O | `hotaru_io_tokio/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| Futures I/O | `hotaru_io_futures/**` | [@JerrySu5379](https://github.com/JerrySu5379) |
| Embedded I/O | `hotaru_io_embedded/**` | [@zkmaojack](https://github.com/zkmaojack) |

## 3. AI declarations

Hotaru supports AI-assisted and AI-copilot development. The human responsible
for a change **must** understand and be able to explain every part of its
design and implementation, regardless of which tools helped produce it. They
must also be able to modify, test, and debug the work without asking an AI
system to reconstruct it for them.

Hotaru does not scan code for an "AI rate," estimate the percentage of code
generated by AI, or use such a percentage as a merge criterion. AI tiers
describe the kind of collaboration, not the amount of generated text. Merge
review instead uses the Update Report, live QA, technical review, and required
CI to assess the design rationale, semantics, risks, compatibility, and the
responsible human's command of the code. This careful process is how Hotaru
makes human accountability and its engineering philosophy visible to other
contributors.

| Tier | Definition |
| --- | --- |
| **Forbidden** | Design, proofs, semantics, and novel logic are human-authored. |
| **Author-Owned** | AI may assist with drafts or completion; the human owns the design and committed work. |
| **Human-Led** | The human writes the structure and load-bearing logic; AI may assist with helpers and boilerplate. |
| **Co-Authored** | AI may assist with design and implementation; the human must fully internalize the result. |

Each Family Maintainer chooses and updates the declarations for components in
their family. When scopes inside one component use different tiers, the more
specific declaration applies.

| Family | Component or scope | Tier |
| --- | --- | --- |
| Core framework | Core `app`, `connection`, `executable`, and `protocol` | **Author-Owned** |
| Core framework | Remaining core contracts and semantics | **Human-Led** |
| Core framework | DSL `endpoint`, `outpoint`, and `middleware` | **Author-Owned** |
| Core framework | Remaining DSL and procedural macros | **Human-Led** |
| Facade and tooling | Routing and URL semantics | **Author-Owned** |
| Facade and tooling | Facade and public feature surface | **Co-Authored** |
| Facade and tooling | CLI and project templates | **Co-Authored** |
| Facade and tooling | Shared utilities | **Human-Led** |
| Protocol implementations | HTTP, CORS, and session middleware | **Human-Led** |
| Protocol implementations | TLS, remaining middleware, and `ahttpm` | **Co-Authored** |
| Protocol implementations | MQTT client and general implementation | **Human-Led** |
| Protocol implementations | MQTT broker and traits | **Co-Authored** |
| Protocol implementations | Experimental protocol integrations | **Co-Authored** |
| Runtime implementations | Tokio and Embassy runtimes | **Co-Authored** |
| I/O implementations | Tokio, Futures, and embedded I/O | **Co-Authored** |

## 4. Eligibility and succession

The Project Maintainer must be an active FDS member. A Family or Component
Maintainer may qualify through either active FDS membership or active PMINE
membership. PMINE membership is independent and does not imply FDS membership.

| Role | Eligibility and appointment |
| --- | --- |
| Project Maintainer | An active FDS member appointed and succeeded under FDS policy |
| Family Maintainer | An active FDS or PMINE member appointed or removed by the Project Maintainer |
| Component Maintainer | An active FDS or PMINE member appointed or removed by the Family Maintainer |
| Reviewer or Steward | Open to trusted contributors; organizational membership is not required |
| Contributor | Open to everyone |

The Project Maintainer follows the
[FDS Charter](https://doc.fds.moe/policies/constitution/). Family and Component
Maintainers are Hotaru technical roles with two independent eligibility paths:
[FDS membership](https://doc.fds.moe/policies/join/) or
[PMINE membership](https://pmine.rs).

A maintainer planning to resign or take leave must arrange a successor or
acting candidate for confirmation by the next higher authority. For an
unexpected vacancy, authority temporarily moves upward. Loss or expiration of
the membership required for a role suspends maintainer authority immediately;
a Family or Component Maintainer remains eligible while actively belonging to
at least one of FDS or PMINE. Every transition must be recorded here and
reflected in code ownership and repository permissions.

This applies the succession principle in the
[FDS Administrator Rules](https://doc.fds.moe/policies/admin/) to Hotaru.
