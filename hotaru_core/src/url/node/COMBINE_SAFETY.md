# Combine Safety — short proof

Covers `Children::combine` (children.rs), `UrlNode::combine` (mod.rs),
`UrlRoot::combine` (../root.rs). Answers two questions: does the recursion
terminate, and can the `PRwLock`s deadlock?

## Setup

- Graph *G*: vertices = `Arc<UrlNode>` allocations, edges = child-table
  entries (+ the root-endpoint slots). Payloads are immutable; only the
  lock-guarded `Children` tables mutate.
- **P1**: both trees are acyclic at call time. Registration only inserts
  fresh or rebind-copied nodes downward; hand-built cycles via the public
  `insert_child` are unsupported (same assumption as `UrlNode::walk`).
- **P2**: build-time merge — no concurrent registration on the same trees.

## Termination

1. Every recursive call `a.combine(&b)` originates as a collision pair one
   child edge deeper in **both** trees than its parent pair.
2. Within one `Children::combine` level, `other`'s children have pairwise
   distinct patterns (keyed table), so a node adopted earlier in the loop
   is never returned by `find` for a later child — the self-side of every
   collision pair predates the loop.
3. Disjoint trees (the intended case): by induction with (2), the self-side
   of every pair is an original `self` node, so pairs correspond 1:1 to
   pattern-paths present in both trees — finitely many, each strictly
   deeper. Depth ≤ min(height(self), height(other)). Merging adds only
   edges from `self` into `other`'s tree, so acyclicity is preserved.
4. Shared-node states (produced by earlier merges): a non-terminating
   recursion would trace an infinite path in finite *G*, forcing a cycle —
   excluded by P1. The `Arc::ptr_eq` short-circuits additionally cut the
   `(x, x)` pairs that earlier merges make common.

*To reproduce:* confirm in children.rs that `all_nodes()` returns an owned
snapshot `Vec`, that `find`/`insert` take transient guards, and that
collisions are returned as an owned `Vec`; then check steps 1–3 against the
recursion in `UrlNode::combine`.

## Deadlock freedom

- Every lock acquisition in the merge path is transient — taken and
  released inside a single helper call: `all_nodes` (other read), `find`
  (self read), `insert` (self write), and the three sequential endpoint
  guards in `UrlRoot::combine`.
- The recursion itself runs guard-free (the collision `Vec` is owned).
- Hence a combining thread only ever *waits* for a lock while *holding*
  none, so it cannot join a hold-and-wait cycle: no self-deadlock, no ABBA
  against a concurrent `b.combine(a)`, and lock re-entrancy never matters.
- The clone hoist in `UrlRoot::combine` is load-bearing: an inline
  `if let Some(o) = other.endpoint.read().clone()` may keep `other`'s read
  guard alive across `self`'s write acquisition (temporary scope is
  edition-dependent), re-admitting ABBA under mutual combine.

## Caveats

- Snapshot semantics: children inserted into `other` mid-merge can be
  missed; the endpoint adopt is check-then-act. Acceptable under P2.
- After a merge the trees share subtrees; later registrations through
  either root mutate the shared tables, visible from both.

Verified by the `combine_*` tests in mod.rs, ../root.rs, and
app/registry.rs.
