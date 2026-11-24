# helix-personal

Personal fork of [Helix](https://github.com/helix-editor/helix) with extra features that
upstream doesn't have. Based on Helix **25.07.1**; features live as small, isolated commits
on top of it.

Helix has no plugin system yet (the Steel PR is still open), so features are implemented
directly in Rust.

## Features

| Feature | Config | Summary |
| ------- | ------ | ------- |
| [Twilight](#twilight) | `[editor.twilight]` | Dim everything outside the code around the cursor, like [twilight.nvim](https://github.com/folke/twilight.nvim) |

### Twilight

Dims text outside the context around the primary cursor in the focused view.

The context is, in order of preference:

1. The **outermost** tree-sitter node around the cursor whose kind is listed in `expand`.
2. The **paragraph** (consecutive non-blank lines) around the cursor, when no such node
   exists or the file has no syntax tree.

Dimmed text is styled with the `ui.twilight` theme scope. If the theme doesn't define it, a
grey foreground is used. Selections and diagnostics stay visible inside dimmed areas.

```toml
# config.toml
[editor.twilight]
enable = false # default
expand = [
  "function_declaration", "method_declaration", "func_literal",   # go
  "function_item", "impl_item",                                    # rust
  "function_definition", "method_definition",                      # python, lua, ...
  "if_statement", "table_constructor",
]
```

```toml
# theme
"ui.twilight" = { fg = "#3d3731", bg = "#161412" } # set bg to the theme background to hide scope colors too
```

Toggle at runtime with `:toggle twilight.enable`, or bind it:

```toml
[keys.normal.space]
t = ":toggle twilight.enable"
```

Differences from twilight.nvim: no `context` line budget and no alpha blending; colors come
from the theme.

Code: `helix-core/src/twilight.rs` (context detection, with tests),
`helix-term/src/ui/editor.rs` (`twilight_highlights`), `helix-view/src/editor.rs` (`TwilightConfig`).

## Install

Requires `git`, `cargo` and a C compiler.

```bash
git clone git@github.com:dabeeduu/helix-personal.git ~/Documents/helix-twilight
cd ~/Documents/helix-twilight
HELIX_DISABLE_AUTO_GRAMMAR_BUILD=1 cargo install --locked --path helix-term

# hx looks for <binary dir>/runtime, so this only affects the fork's binary
ln -sfn "$PWD/runtime" ~/.cargo/bin/runtime

# write grammars into the fork's runtime instead of ~/.config/helix/runtime
CARGO_MANIFEST_DIR=$PWD/helix-term hx --grammar fetch
CARGO_MANIFEST_DIR=$PWD/helix-term hx --grammar build
```

Make sure `~/.cargo/bin` comes before the package manager's `hx` in `PATH`.

The `gotmpl` grammar fails to fetch (its upstream repository is gone). Copy `gotmpl.so` from
an upstream 25.07.1 install into `runtime/grammars/` or ignore it; only Go template files
are affected.

My dotfiles wrap all of this in
[`install-helix-personal`](https://github.com/dabeeduu/dotfiles/blob/main/helix/.config/helix/scripts/install-helix-personal),
which also updates an existing checkout.

Config caveat: a stock Helix rejects unknown config keys, so don't put `[editor.twilight]`
in a `config.toml` shared with it. Disabled is the default, so it can be omitted.

## Development

```bash
export HELIX_DISABLE_AUTO_GRAMMAR_BUILD=1
cargo test -p helix-core --locked --lib twilight
cargo fmt --check
cargo clippy -p helix-core -p helix-view -p helix-term --locked
```

Tests that parse code need the grammars from the install step.

## Adding a feature

1. Add the code with tests (red, green, refactor).
2. Document the config in `book/src/` (`editor.md`, `themes.md`, ...).
3. Add a row to the **Features** table and a section above.
4. Keep commits focused on one feature, so they stay easy to rebase onto a newer upstream.

## Updating from upstream

```bash
git remote add upstream https://github.com/helix-editor/helix.git   # once
git fetch upstream --tags
git rebase <new-upstream-tag>      # or: git merge <new-upstream-tag>
```

Then rebuild with the install steps. Update the base version at the top of this file.

## License

[MPL-2.0](./LICENSE), same as Helix. All credit for the editor goes to the
[Helix contributors](https://github.com/helix-editor/helix/graphs/contributors).
