# Tiny Table

Tiny Table is a simple and customizable table library for Rust.

## Installation

```bash
cargo add tiny-table
```

Or add it manually:

```toml
[dependencies]
tiny-table = "0.1.0"
```

To disable ANSI styling support:

```toml
[dependencies]
tiny-table = { version = "0.1.0", default-features = false }
```

## Examples

### Simple

```rust
use tiny_table::{Cell, Column, Table};

fn main() {
    let mut table = Table::with_columns(vec![
        Column::new("Name").width(15),
        Column::new("Role").width(20),
        Column::new("Status").width(10),
    ]);

    table.add_row(vec![
        Cell::new("Ada Lovelace"),
        Cell::new("Engineer"),
        Cell::new("Active"),
    ]);

    table.add_row(vec![
        Cell::new("Bob"),
        Cell::new("Support"),
        Cell::new("Away"),
    ]);

    println!("{}", table);
}
```

```
┌─────────────────┬──────────────────────┬────────────┐
│ Name            │ Role                 │ Status     │
├─────────────────┼──────────────────────┼────────────┤
│ Ada Lovelace    │ Engineer             │ Active     │
│ Bob             │ Support              │ Away       │
└─────────────────┴──────────────────────┴────────────┘
```

### Styled

```rust
use tiny_table::{Align, Cell, Column, SectionStyle, Table, Trunc};

fn main() {
    let mut table = Table::with_columns(vec![
        Column::new("Name").bright_cyan().bold().width(0.3),
        Column::new("Role").width(0.4).truncate(Trunc::Middle),
        Column::new("Status").bright_yellow().bold().width(0.3),
    ])
    .with_section_style(SectionStyle {
        horiz: "═",
        mid_left: "╞",
        mid_right: "╡",
        mid_joint: "╪",
    })
    .with_separator_style(SectionStyle {
        horiz: "╌",
        mid_joint: "│",
        ..SectionStyle::unicode()
    });

    table.add_section("Team").align(Align::Center);
    table.add_row(vec![
        Cell::new("Ada Lovelace"),
        Cell::new("Principal Engineer"),
        Cell::new("Active").bright_green(),
    ]);

    table.add_separator();
    table.add_row(vec![
        Cell::new("Bob"),
        Cell::new("Support"),
        Cell::new("Away"),
    ]);

    println!("{}", table);
}
```

```
┌────────────────┬────────────────────┬───────────────┐
│ Name           │ Role               │ Status        │
╞════════════════╪══════ Team ════════╪═══════════════╡
│ Ada Lovelace   │ Principal Engineer │ Active        │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌│╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌│╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Bob            │ Support            │ Away          │
└────────────────┴────────────────────┴───────────────┘
```

## Feature Flags

### Style

Adds ANSI styling functions.
