# nitrocop — Agent Reference

Fast Ruby linter in Rust targeting RuboCop compatibility. Uses Prism (`ruby_prism` crate) for parsing.

## Architecture

- `src/cop/` — Cop implementations, organized by department (`layout/`, `lint/`, `style/`, etc.)
- `src/cop/mod.rs` — `Cop` trait definition and `CopRegistry`
- `src/diagnostic.rs` — `Diagnostic` type (severity, location, message)
- `src/parse/source.rs` — `SourceFile` (line offsets, byte-to-line:col conversion)
- `tests/fixtures/cops/<dept>/<cop_name>/` — Test fixtures per cop

## Cop Trait

Every cop implements the `Cop` trait:

```rust
fn name(&self) -> &'static str;                    // e.g., "Style/FrozenStringLiteralComment"
fn interested_node_types(&self) -> &'static [u8];  // Prism node types to visit

// Main detection methods (implement one or more):
fn check_node(&self, source, node, parse_result, config, diagnostics, corrections);  // AST walk
fn check_lines(&self, source, parse_result, config, diagnostics, corrections);       // line-by-line
fn check_source(&self, source, parse_result, config, diagnostics, corrections);      // whole-source
```

`check_node` is called for every AST node whose type is in `interested_node_types()`.
Use `node.as_call_node()`, `node.as_if_node()`, etc. to downcast.

## Prism Node Types — Common Pitfalls

These are the most frequent sources of bugs:

| Parser gem | Prism | Notes |
|-----------|-------|-------|
| `const` | `ConstantReadNode` + `ConstantPathNode` | Simple `Foo` vs qualified `Foo::Bar` — handle BOTH |
| `hash` | `HashNode` + `KeywordHashNode` | Literal `{}` vs keyword args `foo(a: 1)` — handle BOTH |
| `send`/`csend` | `CallNode` | Check `.call_operator()` for safe-navigation `&.` |
| `begin` | `BeginNode` + `StatementsNode` | Explicit `begin..end` vs implicit method body |
| `nil?` in NodePattern | `receiver().is_none()` | Means "child is absent", NOT a `NilNode` literal |
| `super` | `SuperNode` + `ForwardingSuperNode` | `super(args)` vs bare `super` |

### Navigating Parent/Enclosing Nodes

Prism does NOT provide parent pointers. To check what structure encloses a node:
- Check for enclosing blocks by matching node types in `interested_node_types()` and tracking state
- For scope checks: `ProgramNode` (top-level), `ClassNode`, `ModuleNode`, `DefNode`, `BlockNode`
- Special blocks: `PreExecutionNode` (`BEGIN {}`), `PostExecutionNode` (`END {}`)

### Config Access

Cops receive a `CopConfig` with these helpers:
```rust
config.get_bool("KeyName", default)        // bool with default
config.get_str("KeyName", "default")       // &str
config.get_usize("KeyName", default)       // usize
config.get_string_array("KeyName")         // Option<Vec<String>>
config.get_string_hash("KeyName")          // Option<HashMap<String, String>>
```

Keys come from the cop's section in `.rubocop.yml` / vendor `config/default.yml`.

## Test Fixtures

Each cop has `tests/fixtures/cops/<dept>/<cop_name>/offense.rb` and `no_offense.rb`.

**offense.rb** — annotate offenses with `^` markers:
```ruby
x = 1
     ^^ Layout/TrailingWhitespace: Trailing whitespace detected.
```
The `^` characters align with the offending columns. Format: `Department/CopName: message`.

**no_offense.rb** — clean Ruby that should NOT trigger the cop (min 5 non-empty lines).

Run tests: `cargo test --lib -- cop::<dept>::<cop_name>`

## Key Constraints

- `ruby_prism::ParseResult` is `!Send + !Sync` — parsing happens per-thread
- Cop trait is `Send + Sync` — no mutable state on the cop struct
- Edition 2024 (Rust 1.85+)
- Do NOT use `git stash` — commit work-in-progress instead
