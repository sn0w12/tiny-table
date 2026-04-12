use tiny_table::{Align, Cell, Column, ColumnWidth, SectionStyle, Table, Trunc};

fn main() {
    let mut table = Table::with_columns(vec![
        Column::new("Name")
            .bright_cyan()
            .bold()
            .width(ColumnWidth::fill()),
        Column::new("Role").width(0.5).truncate(Trunc::Middle),
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
