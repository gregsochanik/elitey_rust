use colored::*;
use comfy_table::{Cell, CellAlignment, ContentArrangement, Table as Grid};
use comfy_table::{LineStyle, TableStyle};

const UNICODE_STYLE: TableStyle =
    TableStyle::new().header_separator(LineStyle::none().fill('─').junction('─'));
const ASCII_STYLE: TableStyle =
    TableStyle::new().header_separator(LineStyle::none().fill('-').junction('-'));

/// Left-aligned text columns and right-aligned number columns under a header rule
pub struct Table {
    grid: Grid,
}

impl Table {
    pub fn new(columns: &[(&str, CellAlignment)]) -> Self {
        let mut grid = Grid::new();
        let preset = if control::SHOULD_COLORIZE.should_colorize() {
            UNICODE_STYLE
        } else {
            ASCII_STYLE
        };
        grid.load_style(preset)
            .set_content_arrangement(ContentArrangement::Disabled)
            .set_header(columns.iter().map(|(name, _)| Cell::new(name)));
        for (index, (_, alignment)) in columns.iter().enumerate() {
            if let Some(column) = grid.column_mut(index) {
                column.set_cell_alignment(*alignment);
            }
        }
        Self { grid }
    }

    pub fn row<S: ToString>(&mut self, cells: impl IntoIterator<Item = S>) -> &mut Self {
        self.grid
            .add_row(cells.into_iter().map(|cell| cell.to_string()));
        self
    }

    pub fn render(&self) -> String {
        self.grid.to_string()
    }

    pub fn print(&self) {
        println!("{}", self.render());
    }
}

pub const LEFT: CellAlignment = CellAlignment::Left;
pub const RIGHT: CellAlignment = CellAlignment::Right;

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(table: &Table) -> Vec<String> {
        table.render().lines().map(str::to_string).collect()
    }

    #[test]
    fn numbers_are_right_aligned_and_text_left_aligned() {
        let mut table = Table::new(&[("Item", LEFT), ("Price", RIGHT)]);
        table.row(["Food", "5"]).row(["Machinery", "1250"]);

        let lines = lines(&table);

        let columns: Vec<usize> = lines
            .iter()
            .filter(|line| line.contains("Food") || line.contains("Machinery"))
            .map(|line| line.trim_end().len())
            .collect();
        assert_eq!(columns[0], columns[1], "{:?}", lines);
        assert!(lines
            .iter()
            .any(|line| line.contains("Food") && line.trim_end().ends_with("   5")));
        assert!(lines
            .iter()
            .any(|line| line.contains("Machinery") && line.trim_end().ends_with("1250")));
    }

    #[test]
    fn columns_are_as_wide_as_their_longest_cell() {
        let mut table = Table::new(&[("A", LEFT), ("B", LEFT)]);
        table.row(["short", "x"]).row(["a much longer cell", "y"]);

        let lines = lines(&table);
        let positions: Vec<usize> = lines
            .iter()
            .filter(|line| line.trim_end().ends_with('x') || line.trim_end().ends_with('y'))
            .map(|line| line.trim_end().len())
            .collect();
        assert_eq!(positions[0], positions[1], "{:?}", lines);
    }

    #[test]
    fn a_rule_separates_the_header_from_the_rows() {
        let mut table = Table::new(&[("Name", LEFT)]);
        table.row(["Food"]);

        let lines = lines(&table);

        assert!(lines[0].contains("Name"));
        assert!(lines[1].chars().all(|c| c == '-' || c == '─' || c == ' '));
        assert!(lines[2].contains("Food"));
    }

    #[test]
    fn colour_codes_do_not_widen_columns() {
        control::set_override(true);
        let mut table = Table::new(&[("Name", LEFT), ("Value", RIGHT)]);
        table
            .row(["Food".green().to_string(), "5".red().to_string()])
            .row(["Machinery".to_string(), "1250".to_string()]);
        let text = table.render();
        control::unset_override();

        let plain = String::from_utf8(strip_ansi(&text)).unwrap();
        let widths: Vec<usize> = plain
            .lines()
            .filter(|l| l.contains("Food") || l.contains("Machinery"))
            .map(|l| l.trim_end().chars().count())
            .collect();
        assert_eq!(widths[0], widths[1], "{}", plain);
    }

    fn strip_ansi(text: &str) -> Vec<u8> {
        let mut out = Vec::new();
        let mut bytes = text.bytes().peekable();
        while let Some(b) = bytes.next() {
            if b == 0x1b {
                for c in bytes.by_ref() {
                    if c == b'm' {
                        break;
                    }
                }
            } else {
                out.push(b);
            }
        }
        out
    }
}
