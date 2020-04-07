use tabular::{Row, Table};

pub mod accounts;
pub mod dns;
pub mod zones;
pub mod config;

fn table_from_cols(columns: Vec<&str>) -> Table {
    let cols: Vec<&str> = columns.iter().map(|_| "{:<}").collect();
    let spec: &str = &cols.join("    ").to_owned();
    let mut table = Table::new(spec);

    let mut header = Row::new();
    for c in &columns {
        header.add_cell(c);
    }

    table.add_row(header);
    table
}
