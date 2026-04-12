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