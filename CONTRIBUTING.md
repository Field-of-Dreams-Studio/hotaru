# Contributing to Hotaru

Thank you for your interest in contributing to Hotaru. We're excited to build this framework together with the community.

## Development Status

Hotaru is currently in active development (0.8.x). The following areas are still being built:

### In Development

- **Homepage (fds.rs)** - Our official website is under construction
- **Tutorial & Documentation** - Comprehensive guides and examples are being written
- **API Documentation** - Detailed docs for all public APIs
- **Example Projects** - Real-world application examples
- **Backend crate documentation** - Tokio runtime/IO backends now live outside `hotaru_core`, and the feature model needs clear user-facing examples
- **Embedded / no_std support** - Core is being prepared for no_std targets; embedded-io adapters are experimental and real Embassy wiring is deferred

## How You Can Help

We welcome contributions in the following areas:

### Documentation
- Write tutorials for common use cases
- Improve README files and code examples
- Create getting-started guides
- Document best practices and patterns
- Translate documentation to other languages

### Examples
- Build example applications demonstrating Hotaru features
- Create templates for common project types
- Share integration examples with other libraries

### Code
- Fix bugs and improve error messages
- Add tests for uncovered functionality
- Optimize performance
- Implement new features (see our roadmap)

### Community
- Help answer questions in discussions
- Write blog posts or tutorials
- Share your Hotaru projects
- Provide feedback on the API design

## Get Involved

- **GitHub Issues**: https://github.com/Field-of-Dream-Studio/hotaru/issues
- **Discussions**: https://github.com/Field-of-Dream-Studio/hotaru/discussions
- **Email**: redstone@fds.moe
- **Discord Group**: https://discord.gg/Y6b9KRUCux
- **QQ Group**: 860691370
- **Join FDS**: https://forms.office.com/Pages/ResponsePage.aspx?id=DQSIkWdsW0yxEjajBLZtrQAAAAAAAAAAAAMAAC6BwJ5UQ0lQUzdMTjhGR1g3SElLTFdHQUlJV0hFMS4u

## Areas Needing Help

### High Priority
1. **Tutorial Documentation** - Step-by-step guides for:
   - Basic HTTP server setup
   - Middleware creation and usage
   - Session management
   - Custom protocol implementation
   - Custom `TransportSpec` / `RuntimeSpec` implementations
   - Feature selection (`tokio`, `io_futures`, `io_embedded`, `spawn_send`, `spawn_local`)

2. **Homepage Development** - Help build fds.rs:
   - Landing page design
   - Documentation hosting
   - Interactive examples
   - API reference browser

3. **Example Applications**:
   - Blog/CMS system
   - REST API backend
   - Real-time chat application
   - File sharing service
   - Authentication & authorization examples

### Medium Priority
4. **Performance Benchmarks**
   - Compare with other Rust frameworks
   - Identify optimization opportunities
   - Create benchmark suite

5. **Testing**
   - URL routing edge cases
   - Middleware chain testing
   - Integration tests
   - Feature-matrix checks for default Tokio, no-default facade builds, and core-only builds

## Governance and PR requirements

Substantive changes go through Hotaru's governance process — an Update Report and a live QA, with AI-collaboration tiers declared per component. See [GOVERNANCE.md](./GOVERNANCE.md) for the roles, PR routes, and tier definitions before opening a PR. Contributors who prefer not to run the canonical PR themselves may ask a Component or Family Maintainer to stage the change instead; in that case the maintainer handles the Update Report and live QA, and the contributor is still credited for authored work.

## Contribution Guidelines

1. **Fork the repository** and create a feature branch
2. **Write clear commit messages** describing your changes
3. **Add tests** for new functionality
4. **Update documentation** if you change APIs
5. **Follow the existing code style** (run `cargo fmt` on the files you touch)
6. **Ensure the workspace builds** (`cargo check --workspace`) and tests pass (`cargo test`)
7. **Open a Pull Request** with a clear description

> Note: use `cargo fmt` on changed files rather than `cargo fmt --all`, which
> produces unrelated workspace-wide formatting noise. When testing example
> crates, prefer the `hotaru build` / `hotaru run` CLI commands so templates and
> static assets are copied correctly.
>
> Internal Hotaru crate dependencies should use exact version pins such as
> `version = "=0.8.3"` during release-prep updates. Third-party dependencies
> should keep normal semver requirements unless there is a specific reason to
> pin them.

## Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Run `cargo clippy` and fix warnings
- Add doc comments (`///`) for public APIs
- Write descriptive variable and function names

For framework code style and formatting requirements, see
[CONTRIBUTOR_STYLE.md](./CONTRIBUTOR_STYLE.md).

## Project Roadmap

### 0.8.3 (Current)
- Core/backend split: Tokio runtime and IO backends live in sibling crates (`hotaru_rt_tokio`, `hotaru_io_tokio`, `hotaru_io_futures`, `hotaru_io_embedded`)
- `hotaru_core` keeps only platform/sync (`std` / `embedded`) and task-mobility (`spawn_send` / `spawn_local`) feature axes
- no_std / embedded groundwork (experimental; real Embassy wiring deferred)
- HTTP/TLS hardening and documentation

### 0.9.0
- UDP support
- Performance optimization

### 1.0.0
- API stability guarantee
- Complete documentation
- Production deployment guides

## License

By contributing to Hotaru, you agree that your contributions will be licensed under the MIT License.

## Thank You

Your contributions make Hotaru better for everyone. Whether you fix a typo, write documentation, or implement a major feature, every contribution is valuable.

Let's build something great together.
