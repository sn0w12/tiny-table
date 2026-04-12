use simple_table::{Align, Cell, Column, SectionStyle, Table, Trunc};

fn main() {
    let header_style = SectionStyle {
        horiz: "═",
        mid_left: "╞",
        mid_right: "╡",
        mid_joint: "╪",
    };

    let separator_style = SectionStyle {
        horiz: "╌",
        mid_joint: "│",
        ..SectionStyle::unicode()
    };

    let mut table = Table::with_columns(vec![
        Column::new("Name").bright_cyan().bold().width(0.18),
        Column::new("Role")
            .width(0.22)
            .truncate(Trunc::Middle)
            .align(Align::Center),
        Column::new("Dept").bright_blue().width(0.12),
        Column::new("Location").width(0.14).truncate(Trunc::Middle),
        Column::new("Status").bright_yellow().bold().width(0.12),
        Column::new("Notes")
            .bright_magenta()
            .width(0.22)
            .truncate(Trunc::NewLine)
            .align(Align::Right),
    ])
    .with_section_style(header_style)
    .with_separator_style(separator_style);

    table.add_section("Employee directory").align(Align::Center);

    table.add_row(vec![
        Cell::new("Alice Johnson"),
        Cell::new("Senior Product Manager"),
        Cell::new("Product"),
        Cell::new("San Francisco, California"),
        Cell::new("Active"),
        Cell::new("Loves long-term strategy, cross-functional leadership, and coffee breaks"),
    ]);

    table.add_row(vec![
        Cell::new("Bob").bright_green().bold(),
        Cell::new("Backend engineer"),
        Cell::new("Platform"),
        Cell::new("Berlin"),
        Cell::new("On leave").bright_red().underline(),
        Cell::new("Writes services in Rust and maintains the CI pipeline").truncate(Trunc::NewLine),
    ]);

    table.add_separator();

    table.add_row(vec![
        Cell::new("Cynthia \"CJ\" Lee"),
        Cell::new("UI/UX Designer").truncate(Trunc::Start),
        Cell::new("Design"),
        Cell::new("New York").truncate(Trunc::Middle),
        Cell::new("Active"),
        Cell::new("Experimenting with animation and accessibility features"),
    ]);

    table.add_row(vec![
        Cell::new("Dmitri"),
        Cell::new("Customer support"),
        Cell::new("Operations"),
        Cell::new("Moscow"),
        Cell::new("Active"),
        Cell::new("Handles tickets, escalation, and knowledge-base updates"),
    ]);

    table.add_row(vec![
        Cell::new("Elena"),
        Cell::new("Data scientist"),
        Cell::new("Analytics"),
        Cell::new("São Paulo"),
        Cell::new("Pending"),
        Cell::new("Onboarding a new data model for daily reporting").truncate(Trunc::NewLine),
    ]);

    println!("{}", table);
}
