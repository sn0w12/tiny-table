//! Table rendering primitives and formatting logic.
//!
//! The crate root re-exports the main types from this module, so most users
//! will interact with [`Table`], [`Column`], [`Cell`], [`Trunc`], and
//! [`Align`].
//!
//! The renderer works in three steps:
//!
//! 1. Measure content width for headers and rows.
//! 2. Apply column width rules, truncation, and wrapping.
//! 3. Emit a bordered table using the active terminal width when available.
//!
//! The public API is intentionally small. `Table` owns the rows, `Column`
//! defines the schema, and `Cell` gives you per-value overrides when needed.

#[cfg(feature = "style")]
pub use crate::color::{Color, CustomColor};
use std::cmp::Ordering;
use std::collections::HashMap;
use terminal_size::{Width, terminal_size};

#[cfg(feature = "style")]
mod style;
mod text;

#[cfg(feature = "style")]
use style::{StyleAction, apply_style_actions, impl_style_methods};
use text::{layout_line, split_lines, strip_ansi, truncate_line, visible_len};

const ANSI_RESET: &str = "\x1b[0m";

/// Characters used to draw table borders and joints.
///
/// The renderer currently uses the Unicode preset from [`TableStyle::unicode`].
/// Section and separator rows can also use their own `TableStyle` snapshot,
/// which lets you mix multiple visual themes in a single table.
/// This type exists so the border characters are grouped in one place if you
/// want to experiment with alternate themes or extend the renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableStyle {
    /// Top-left corner character.
    pub top_left: &'static str,
    /// Top-right corner character.
    pub top_right: &'static str,
    /// Bottom-left corner character.
    pub bottom_left: &'static str,
    /// Bottom-right corner character.
    pub bottom_right: &'static str,
    /// Horizontal border character.
    pub horiz: &'static str,
    /// Vertical border character.
    pub vert: &'static str,
    /// Top border joint between columns.
    pub top_joint: &'static str,
    /// Left border character for section separators and middle rules.
    pub mid_left: &'static str,
    /// Right border character for section separators and middle rules.
    pub mid_right: &'static str,
    /// Joint character for interior separators.
    pub mid_joint: &'static str,
    /// Bottom border joint between columns.
    pub bottom_joint: &'static str,
}

impl TableStyle {
    /// Return the default Unicode border style.
    pub fn unicode() -> Self {
        TableStyle {
            top_left: "┌",
            top_right: "┐",
            bottom_left: "└",
            bottom_right: "┘",
            horiz: "─",
            vert: "│",
            top_joint: "┬",
            mid_left: "├",
            mid_right: "┤",
            mid_joint: "┼",
            bottom_joint: "┴",
        }
    }

    /// Convert a [`SectionStyle`] into a [`TableStyle`] by filling in the missing fields
    pub fn from_section_style(section_style: SectionStyle) -> Self {
        TableStyle {
            horiz: section_style.horiz,
            mid_left: section_style.mid_left,
            mid_right: section_style.mid_right,
            mid_joint: section_style.mid_joint,
            ..TableStyle::unicode()
        }
    }
}

/// Characters used to overwrite section and separator styles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SectionStyle {
    /// Horizontal border character.
    pub horiz: &'static str,
    /// Left border character for section separators and middle rules.
    pub mid_left: &'static str,
    /// Right border character for section separators and middle rules.
    pub mid_right: &'static str,
    /// Joint character for interior separators.
    pub mid_joint: &'static str,
}

impl SectionStyle {
    /// Return the default Unicode border style for sections and separators.
    pub fn unicode() -> Self {
        SectionStyle::from_table_style(TableStyle::unicode())
    }

    /// Convert a [`TableStyle`] into a [`SectionStyle`] by taking the relevant fields
    pub fn from_table_style(table_style: TableStyle) -> Self {
        SectionStyle {
            horiz: table_style.horiz,
            mid_left: table_style.mid_left,
            mid_right: table_style.mid_right,
            mid_joint: table_style.mid_joint,
        }
    }
}

/// How to handle content that does not fit inside the available column width.
///
/// Truncation is applied after the column width is resolved. If you need the
/// full content to remain visible, use [`Trunc::NewLine`] to wrap onto more
/// lines instead of clipping the text.
///
/// # Examples
///
/// ```rust
/// use tiny_table::{Cell, Trunc};
///
/// let cell = Cell::new("abcdefghij").truncate(Trunc::Middle);
/// ```
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Trunc {
    /// Keep the beginning of the text and add an ellipsis at the end.
    #[default]
    End,
    /// Keep the end of the text and add an ellipsis at the start.
    Start,
    /// Keep the start and end of the text with an ellipsis in the middle.
    Middle,
    /// Wrap onto multiple lines instead of truncating.
    NewLine,
}

/// Alignment options for any content supporting alignment.
///
/// # Examples
///
/// ```rust
/// use tiny_table::{Align, Table};
///
/// let mut table = Table::new();
/// table.add_section("Team").align(Align::Right);
/// ```
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Align {
    /// Center the content within the available width.
    #[default]
    Center,
    /// Place the content toward the left side of the available width.
    Left,
    /// Place the content toward the right side of the available width.
    Right,
}

/// Width policy for a table column.
///
/// `Auto` keeps the column at content width unless a column-specific width is
/// applied. `Fixed` uses an exact terminal-cell width. `Fraction` divides the
/// remaining terminal width proportionally across all fractional columns.
///
/// # Examples
///
/// ```rust
/// use tiny_table::ColumnWidth;
///
/// let fixed = ColumnWidth::fixed(12);
/// let fraction = ColumnWidth::fraction(0.5);
/// assert_eq!(fixed, ColumnWidth::Fixed(12));
/// assert_eq!(fraction, ColumnWidth::Fraction(0.5));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ColumnWidth {
    /// Size the column from its content and any explicit constraints.
    #[default]
    Auto,
    /// Use an exact width in terminal cells.
    Fixed(usize),
    /// Allocate a fraction of the remaining available width.
    Fraction(f64),
}

impl ColumnWidth {
    /// Create a fixed-width column.
    pub fn fixed(width: usize) -> Self {
        Self::Fixed(width)
    }

    /// Create a fractional-width column.
    pub fn fraction(fraction: f64) -> Self {
        Self::Fraction(fraction)
    }
}

macro_rules! impl_column_width_from_int {
    ($($ty:ty),* $(,)?) => {
        $(impl From<$ty> for ColumnWidth {
            fn from(width: $ty) -> Self {
                Self::Fixed(width.max(0) as usize)
            }
        })*
    };
}

impl_column_width_from_int!(
    usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128
);

impl From<f32> for ColumnWidth {
    fn from(fraction: f32) -> Self {
        Self::Fraction(fraction as f64)
    }
}

impl From<f64> for ColumnWidth {
    fn from(fraction: f64) -> Self {
        Self::Fraction(fraction)
    }
}

/// Selector used to apply styling to a column.
///
/// [`Index`](ColumnTarget::Index) is zero-based. [`Header`](ColumnTarget::Header)
/// matches the exact header text supplied to [`Column::new`].
///
/// # Examples
///
/// ```rust
/// use tiny_table::ColumnTarget;
///
/// assert_eq!(ColumnTarget::from(0usize), ColumnTarget::Index(0));
/// assert_eq!(ColumnTarget::from("Name"), ColumnTarget::Header("Name".to_string()));
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ColumnTarget {
    /// Target a column by zero-based index.
    Index(usize),
    /// Target a column by its header text.
    Header(String),
}

impl From<usize> for ColumnTarget {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

impl From<&str> for ColumnTarget {
    fn from(header: &str) -> Self {
        Self::Header(header.to_string())
    }
}

impl From<String> for ColumnTarget {
    fn from(header: String) -> Self {
        Self::Header(header)
    }
}

/// A column definition that bundles the header text with default styling.
///
/// Use this when you already know your schema and want the table to inherit
/// width, truncation, and color defaults from the column definition.
///
/// # Examples
///
/// ```rust
/// use tiny_table::{Column, Color, Trunc};
///
/// let column = Column::new("Status")
///     .color(Color::BrightGreen)
///     .width(12)
///     .truncate(Trunc::End);
/// ```
pub struct Column {
    header: Cell,
    style: ColumnStyle,
}

impl Column {
    /// Create a new column definition from a header value.
    pub fn new(header: impl Into<Cell>) -> Self {
        Self {
            header: header.into(),
            style: ColumnStyle::default(),
        }
    }

    /// Set the preferred width for this column.
    pub fn width(mut self, width: impl Into<ColumnWidth>) -> Self {
        self.style.width = Some(width.into());
        self
    }

    /// Alias for [`Column::width`].
    pub fn max_width(self, width: impl Into<ColumnWidth>) -> Self {
        self.width(width)
    }

    /// Set the default truncation strategy used by this column.
    pub fn truncate(mut self, truncation: Trunc) -> Self {
        self.style.truncation = Some(truncation);
        self
    }

    /// Set the default text alignment for this column.
    pub fn align(mut self, align: Align) -> Self {
        self.style.align = Some(align);
        self
    }
}

#[cfg(feature = "style")]
impl_style_methods!(Column, |mut column: Column, action| {
    column.style.styles.push(action);
    column
});

#[derive(Clone, Debug, Default)]
struct ColumnStyle {
    #[cfg(feature = "style")]
    styles: Vec<StyleAction>,
    width: Option<ColumnWidth>,
    truncation: Option<Trunc>,
    align: Option<Align>,
}

impl ColumnStyle {
    fn merge(&mut self, other: &ColumnStyle) {
        #[cfg(feature = "style")]
        self.styles.extend_from_slice(&other.styles);

        if other.width.is_some() {
            self.width = other.width;
        }

        if other.truncation.is_some() {
            self.truncation = other.truncation;
        }

        if other.align.is_some() {
            self.align = other.align;
        }
    }
}

/// A single table cell with optional styling overrides.
///
/// Cells are usually created from strings and then used in [`Table::add_row`].
/// Any color or truncation set on the cell takes priority over the owning
/// column's defaults.
///
/// # Examples
///
/// ```rust
/// use tiny_table::{Cell, Color, Trunc};
///
/// let cell = Cell::new("warning")
///     .color(Color::BrightRed)
///     .truncate(Trunc::Middle);
/// ```
pub struct Cell {
    content: String,
    #[cfg(feature = "style")]
    styles: Vec<StyleAction>,
    truncation: Option<Trunc>,
    align: Option<Align>,
}

struct PreparedCell {
    lines: Vec<String>,
    align: Align,
}

enum TableRow {
    Cells(Vec<Cell>),
    Section(SectionRow),
}

enum PreparedRow {
    Cells(Vec<PreparedCell>),
    Section(SectionRow),
}

#[derive(Clone, Debug)]
struct SectionRow {
    title: String,
    align: Align,
    style: TableStyle,
}

/// Builder returned by [`Table::add_section`] and [`Table::add_separator`].
pub struct SectionBuilder<'a> {
    table: &'a mut Table,
    row_index: usize,
}

/// Builder returned by [`Table::column`] for per-column styling.
///
/// Use this to override color, width, or truncation for one column without
/// changing the schema-wide defaults defined by [`Column`].
pub struct ColumnBuilder<'a> {
    table: &'a mut Table,
    target: ColumnTarget,
}

impl Cell {
    /// Create a new cell from display content.
    pub fn new(content: impl ToString) -> Self {
        Self {
            content: content.to_string(),
            #[cfg(feature = "style")]
            styles: Vec::new(),
            truncation: None,
            align: None,
        }
    }

    /// Set a truncation mode for this cell, overriding any column default.
    #[must_use]
    pub fn truncate(mut self, truncation: Trunc) -> Self {
        self.truncation = Some(truncation);
        self
    }

    /// Set the text alignment for this cell, overriding any column default.
    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }
}

#[cfg(feature = "style")]
impl_style_methods!(Cell, |mut cell: Cell, action| {
    cell.styles.push(action);
    cell
});

impl From<&str> for Cell {
    fn from(content: &str) -> Self {
        Self::new(content)
    }
}

impl From<String> for Cell {
    fn from(content: String) -> Self {
        Self::new(content)
    }
}

/// Main table type for terminal output.
///
/// A table owns its headers, rows, and column styling. Content is measured at
/// render time, so the output can adapt to the current terminal width without
/// manual layout code from the caller.
///
/// # Typical Flow
///
/// 1. Build the schema with [`Table::with_columns`] or [`Table::new`].
/// 2. Add rows with [`Table::add_row`].
/// 3. Adjust columns with [`Table::column`] if you need per-column overrides.
/// 4. Call [`Table::render`] or print the table directly.
///
/// # Example
///
/// ```rust
/// use tiny_table::{Cell, Column, Align, Table, Trunc};
///
/// let mut table = Table::with_columns(vec![
///     Column::new("Name").width(0.35),
///     Column::new("Role").truncate(Trunc::Middle),
///     Column::new("Status"),
/// ]);
///
/// table.add_section("Team").align(Align::Center);
/// table.add_row(vec![
///     Cell::new("Ada Lovelace"),
///     Cell::new("Principal Engineer"),
///     Cell::new("Active"),
/// ]);
///
/// let rendered = table.render();
/// assert!(rendered.contains("Name"));
/// assert!(rendered.contains("Ada Lovelace"));
/// ```
pub struct Table {
    headers: Vec<Cell>,
    rows: Vec<TableRow>,
    column_defaults: Vec<ColumnStyle>,
    column_overrides: HashMap<ColumnTarget, ColumnStyle>,
    style: TableStyle,
    section_style: Option<SectionStyle>,
    separator_style: Option<SectionStyle>,
}

impl Table {
    /// Create an empty table using the default Unicode border style.
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            column_defaults: Vec::new(),
            column_overrides: HashMap::new(),
            style: TableStyle::unicode(),
            section_style: None,
            separator_style: None,
        }
    }

    /// Build a table from an explicit column schema.
    ///
    /// Each [`Column`] contributes the header text and the default formatting
    /// for that column.
    pub fn with_columns(columns: impl IntoIterator<Item = Column>) -> Self {
        let mut table = Self::new();
        table.set_columns(columns);
        table
    }

    /// Replace the default border style for this table.
    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the default style used for all section rows added with [`Table::add_section`].
    ///
    /// Individual sections can still override this by calling `.style()` on the
    /// [`SectionBuilder`] returned by [`Table::add_section`].
    pub fn with_section_style(mut self, style: SectionStyle) -> Self {
        self.section_style = Some(style);
        self
    }

    /// Set the default style used for all separator rows added with [`Table::add_separator`].
    ///
    /// Individual separators can still override this by calling `.style()` on the
    /// [`SectionBuilder`] returned by [`Table::add_separator`].
    pub fn with_separator_style(mut self, style: SectionStyle) -> Self {
        self.separator_style = Some(style);
        self
    }

    /// Replace the table schema with a new set of columns.
    ///
    /// This clears existing headers and column-specific overrides before the
    /// new schema is applied.
    pub fn set_columns(&mut self, columns: impl IntoIterator<Item = Column>) {
        let (headers, column_defaults): (Vec<_>, Vec<_>) = columns
            .into_iter()
            .map(|Column { header, style }| (header, style))
            .unzip();

        self.headers = headers;
        self.column_defaults = column_defaults;
        self.column_overrides.clear();
    }

    /// Add a data row.
    ///
    /// Short rows are padded with empty cells. If a row has more cells than the
    /// header list, the table expands to fit the additional columns.
    pub fn add_row(&mut self, row: Vec<Cell>) {
        self.rows.push(TableRow::Cells(row));
    }

    /// Add a full-width section separator inside the table.
    ///
    /// Section rows span the entire table width and are useful for grouping
    /// related data visually. The returned builder currently lets you choose
    /// the label alignment.
    ///
    /// If a default section style was set with [`Table::with_section_style`], it is
    /// applied automatically. Call `.style()` on the returned builder to override it.
    pub fn add_section(&mut self, title: impl ToString) -> SectionBuilder<'_> {
        let row_index = self.rows.len();
        let style = match self.section_style {
            Some(s) => TableStyle::from_section_style(s),
            None => self.style,
        };
        self.rows.push(TableRow::Section(SectionRow {
            title: title.to_string(),
            align: Align::Center,
            style,
        }));

        SectionBuilder {
            table: self,
            row_index,
        }
    }

    /// Add a full-width separator row with no label.
    ///
    /// If a default separator style was set with [`Table::with_separator_style`], it is
    /// applied automatically. Call `.style()` on the returned builder to override it.
    pub fn add_separator(&mut self) -> SectionBuilder<'_> {
        let row_index = self.rows.len();
        let style = match self.separator_style {
            Some(s) => TableStyle::from_section_style(s),
            None => self.style,
        };
        self.rows.push(TableRow::Section(SectionRow {
            title: String::new(),
            align: Align::Center,
            style,
        }));

        SectionBuilder {
            table: self,
            row_index,
        }
    }

    /// Configure a column using either its zero-based index or exact header text.
    pub fn column<T: Into<ColumnTarget>>(&mut self, target: T) -> ColumnBuilder<'_> {
        ColumnBuilder {
            table: self,
            target: target.into(),
        }
    }

    /// Print the formatted table to standard output.
    #[allow(dead_code)]
    pub fn print(&self) {
        for line in self.render_lines() {
            println!("{line}");
        }
    }

    /// Render the formatted table as a single string.
    ///
    /// The returned string includes ANSI color codes when styling is applied.
    /// Fractional widths are resolved using the current terminal size when it
    /// can be detected.
    pub fn render(&self) -> String {
        self.render_lines().join("\n")
    }

    fn column_style_mut(&mut self, target: ColumnTarget) -> &mut ColumnStyle {
        self.column_overrides.entry(target).or_default()
    }

    fn column_style(&self, col: usize) -> ColumnStyle {
        let mut style = self.column_defaults.get(col).cloned().unwrap_or_default();

        if let Some(header) = self.headers.get(col)
            && let Some(header_style) = self
                .column_overrides
                .get(&ColumnTarget::Header(strip_ansi(&header.content)))
        {
            style.merge(header_style);
        }

        if let Some(index_style) = self.column_overrides.get(&ColumnTarget::Index(col)) {
            style.merge(index_style);
        }

        style
    }

    fn prepare_cell(
        &self,
        cell: Option<&Cell>,
        column_style: &ColumnStyle,
        width: usize,
        #[cfg_attr(not(feature = "style"), allow(unused_variables))] is_header: bool,
    ) -> PreparedCell {
        let raw = cell.map(|c| c.content.as_str()).unwrap_or("");
        let truncation = cell
            .and_then(|c| c.truncation)
            .or(column_style.truncation)
            .unwrap_or(Trunc::End);
        let align = cell
            .and_then(|c| c.align)
            .or(column_style.align)
            .unwrap_or(Align::Left);

        #[cfg(feature = "style")]
        let styled = {
            // Merge styles: column first, then cell overrides (last color wins in
            // the flat ANSI prefix built by apply_style_actions).
            let mut all_styles = column_style.styles.clone();
            if let Some(cell) = cell {
                all_styles.extend_from_slice(&cell.styles);
            }
            if is_header {
                all_styles.push(StyleAction::Bold);
            }
            apply_style_actions(raw, &all_styles)
        };
        #[cfg(not(feature = "style"))]
        let styled = raw.to_string();

        let lines = split_lines(&styled)
            .into_iter()
            .flat_map(|line| layout_line(&line, Some(width), truncation))
            .collect();

        PreparedCell { lines, align }
    }

    fn prepare_row(
        &self,
        row: &[Cell],
        col_widths: &[usize],
        column_styles: &[ColumnStyle],
        is_header: bool,
    ) -> Vec<PreparedCell> {
        col_widths
            .iter()
            .zip(column_styles)
            .enumerate()
            .map(|(col, (&width, style))| self.prepare_cell(row.get(col), style, width, is_header))
            .collect()
    }

    fn collect_content_widths(&self, col_count: usize) -> Vec<usize> {
        let mut widths = vec![0usize; col_count];

        let all_rows =
            std::iter::once(self.headers.as_slice()).chain(self.rows.iter().filter_map(|row| {
                match row {
                    TableRow::Cells(cells) => Some(cells.as_slice()),
                    TableRow::Section(_) => None,
                }
            }));

        for row in all_rows {
            for (col, cell) in row.iter().enumerate() {
                for line in split_lines(&cell.content) {
                    widths[col] = widths[col].max(visible_len(&line));
                }
            }
        }

        widths
    }

    fn resolve_column_widths(
        &self,
        content_widths: &[usize],
        column_styles: &[ColumnStyle],
        terminal_width: Option<usize>,
    ) -> Vec<usize> {
        let mut widths = content_widths.to_vec();
        let mut fraction_columns = Vec::new();
        let mut reserved_width = 0usize;

        for (col, style) in column_styles.iter().enumerate() {
            match style.width.unwrap_or_default() {
                ColumnWidth::Auto => {
                    reserved_width += widths[col];
                }
                ColumnWidth::Fixed(width) => {
                    widths[col] = width;
                    reserved_width += width;
                }
                ColumnWidth::Fraction(fraction) => {
                    fraction_columns.push((col, fraction.max(0.0)));
                }
            }
        }

        let Some(terminal_width) = terminal_width else {
            return widths;
        };

        let table_overhead = (3 * widths.len()) + 1;
        let available_content_width = terminal_width.saturating_sub(table_overhead);
        let remaining_width = available_content_width.saturating_sub(reserved_width);

        if fraction_columns.is_empty() {
            return widths;
        }

        let total_fraction: f64 = fraction_columns.iter().map(|(_, fraction)| *fraction).sum();
        if total_fraction <= f64::EPSILON {
            for (col, _) in fraction_columns {
                widths[col] = 0;
            }

            return widths;
        }

        let mut remainders = Vec::with_capacity(fraction_columns.len());
        let mut assigned = 0usize;

        for (col, fraction) in fraction_columns {
            let exact = (remaining_width as f64) * fraction / total_fraction;
            let width = exact.floor() as usize;
            widths[col] = width;
            assigned += width;
            remainders.push((col, exact - width as f64));
        }

        let mut leftover = remaining_width.saturating_sub(assigned);
        remainders.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(Ordering::Equal));

        for (col, _) in remainders {
            if leftover == 0 {
                break;
            }

            widths[col] += 1;
            leftover -= 1;
        }

        widths
    }

    fn column_count(&self) -> usize {
        let max_row_len = self
            .rows
            .iter()
            .filter_map(|row| match row {
                TableRow::Cells(cells) => Some(cells.len()),
                TableRow::Section(_) => None,
            })
            .max()
            .unwrap_or(0);

        self.headers.len().max(max_row_len)
    }

    fn row_height(cells: &[PreparedCell]) -> usize {
        cells.iter().map(|cell| cell.lines.len()).max().unwrap_or(1)
    }

    fn rule_line(
        &self,
        style: &TableStyle,
        left: &str,
        joint: &str,
        right: &str,
        col_widths: &[usize],
    ) -> String {
        let h = style.horiz;
        let join = format!("{}{}{}", h, joint, h);
        let inner = col_widths
            .iter()
            .map(|&width| h.repeat(width))
            .collect::<Vec<_>>()
            .join(&join);

        format!("{}{}{}{}{}", left, h, inner, h, right)
    }

    fn push_row_lines(
        &self,
        lines: &mut Vec<String>,
        cells: &[PreparedCell],
        col_widths: &[usize],
    ) {
        for line_idx in 0..Self::row_height(cells) {
            lines.push(self.render_row_line(cells, line_idx, col_widths));
        }
    }

    fn render_row_line(
        &self,
        row: &[PreparedCell],
        line_idx: usize,
        col_widths: &[usize],
    ) -> String {
        let vertical = self.style.vert;
        let rendered_cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(col, cell)| {
                let raw = cell.lines.get(line_idx).map(String::as_str).unwrap_or("");
                let padding = col_widths[col].saturating_sub(visible_len(raw));
                match cell.align {
                    Align::Left => format!("{}{}", raw, " ".repeat(padding)),
                    Align::Right => format!("{}{}", " ".repeat(padding), raw),
                    Align::Center => {
                        let left_pad = padding / 2;
                        let right_pad = padding - left_pad;
                        format!("{}{}{}", " ".repeat(left_pad), raw, " ".repeat(right_pad))
                    }
                }
            })
            .collect();

        format!(
            "{} {} {}",
            vertical,
            rendered_cells.join(&format!(" {} ", vertical)),
            vertical
        )
    }

    fn render_section_line(&self, section: &SectionRow, col_widths: &[usize]) -> String {
        let style = &section.style;

        if section.title.trim().is_empty() {
            return self.rule_line(
                style,
                style.mid_left,
                style.mid_joint,
                style.mid_right,
                col_widths,
            );
        }

        let total_inner = col_widths.iter().sum::<usize>() + 3 * col_widths.len() - 1;
        let label = truncate_line(
            &format!(" {} ", section.title),
            Some(total_inner),
            Trunc::End,
        );
        let label_len = label.chars().count();
        let remaining = total_inner.saturating_sub(label_len);

        let left_fill = match section.align {
            Align::Left => 1,
            Align::Center => remaining / 2,
            Align::Right => remaining.saturating_sub(1),
        };

        let mut inner: Vec<char> = style.horiz.repeat(total_inner).chars().collect();
        let joint = style.mid_joint.chars().next().unwrap_or('┼');
        let mut cursor = 1;

        for &w in col_widths.iter().take(col_widths.len().saturating_sub(1)) {
            cursor += w + 1;
            if cursor < inner.len() {
                inner[cursor] = joint;
            }
            cursor += 2;
        }

        let prefix: String = inner[..left_fill].iter().collect();
        let suffix: String = inner[left_fill + label_len..].iter().collect();

        #[cfg(feature = "style")]
        let bold_label = format!("\x1b[1m{}{}", label, ANSI_RESET);
        #[cfg(not(feature = "style"))]
        let bold_label = label;

        format!(
            "{}{}{}{}{}",
            style.mid_left, prefix, bold_label, suffix, style.mid_right
        )
    }

    fn render_lines_with_terminal_width(&self, terminal_width: Option<usize>) -> Vec<String> {
        let col_count = self.column_count();
        if col_count == 0 {
            return Vec::new();
        }

        let column_styles: Vec<ColumnStyle> =
            (0..col_count).map(|col| self.column_style(col)).collect();
        let content_widths = self.collect_content_widths(col_count);
        let col_widths =
            self.resolve_column_widths(&content_widths, &column_styles, terminal_width);

        let prepared_header = (!self.headers.is_empty())
            .then(|| self.prepare_row(&self.headers, &col_widths, &column_styles, true));
        let prepared_rows: Vec<PreparedRow> = self
            .rows
            .iter()
            .map(|row| match row {
                TableRow::Cells(cells) => {
                    PreparedRow::Cells(self.prepare_row(cells, &col_widths, &column_styles, false))
                }
                TableRow::Section(section) => PreparedRow::Section(section.clone()),
            })
            .collect();

        let mut lines = Vec::new();

        lines.push(self.rule_line(
            &self.style,
            self.style.top_left,
            self.style.top_joint,
            self.style.top_right,
            &col_widths,
        ));

        if let Some(header) = prepared_header.as_ref() {
            self.push_row_lines(&mut lines, header, &col_widths);

            if prepared_rows.is_empty()
                || !matches!(prepared_rows.first(), Some(PreparedRow::Section(_)))
            {
                lines.push(self.rule_line(
                    &self.style,
                    self.style.mid_left,
                    self.style.mid_joint,
                    self.style.mid_right,
                    &col_widths,
                ));
            }
        }

        for row in &prepared_rows {
            match row {
                PreparedRow::Cells(cells) => self.push_row_lines(&mut lines, cells, &col_widths),
                PreparedRow::Section(section) => {
                    lines.push(self.render_section_line(section, &col_widths))
                }
            }
        }

        lines.push(self.rule_line(
            &self.style,
            self.style.bottom_left,
            self.style.bottom_joint,
            self.style.bottom_right,
            &col_widths,
        ));

        lines
    }

    fn render_lines(&self) -> Vec<String> {
        self.render_lines_with_terminal_width(
            terminal_size().map(|(Width(width), _)| width as usize),
        )
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.render_lines() {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl<'a> SectionBuilder<'a> {
    /// Set the alignment for the section label.
    pub fn align(self, align: Align) -> Self {
        if let Some(TableRow::Section(section)) = self.table.rows.get_mut(self.row_index) {
            section.align = align;
        }

        self
    }

    /// Set the border style used when rendering this section or separator.
    pub fn style(self, style: SectionStyle) -> Self {
        if let Some(TableRow::Section(section)) = self.table.rows.get_mut(self.row_index) {
            section.style = TableStyle::from_section_style(style);
        }

        self
    }
}

impl<'a> ColumnBuilder<'a> {
    /// Set the default color for the selected column.
    #[cfg(feature = "style")]
    pub fn color(self, color: Color) -> Self {
        self.table
            .column_style_mut(self.target.clone())
            .styles
            .push(StyleAction::Color(color));
        self
    }

    /// Set the preferred width for the selected column.
    pub fn width(self, width: impl Into<ColumnWidth>) -> Self {
        self.table.column_style_mut(self.target.clone()).width = Some(width.into());
        self
    }

    /// Alias for [`ColumnBuilder::width`].
    pub fn max_width(self, width: impl Into<ColumnWidth>) -> Self {
        self.width(width)
    }

    /// Set the truncation strategy for the selected column.
    pub fn truncate(self, truncation: Trunc) -> Self {
        self.table.column_style_mut(self.target.clone()).truncation = Some(truncation);
        self
    }

    /// Set the text alignment for the selected column.
    pub fn align(self, align: Align) -> Self {
        self.table.column_style_mut(self.target.clone()).align = Some(align);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::text::strip_ansi;
    use super::*;
    #[cfg(feature = "style")]
    use crate::Color::BrightBlack;

    impl Table {
        fn render_lines_for_test(&self, terminal_width: Option<usize>) -> Vec<String> {
            self.render_lines_with_terminal_width(terminal_width)
        }
    }

    #[cfg(feature = "style")]
    #[test]
    fn cell_builders_are_chainable() {
        let cell = Cell::new("value")
            .color(BrightBlack)
            .truncate(Trunc::Middle);

        assert!(matches!(
            cell.styles.as_slice(),
            [StyleAction::Color(BrightBlack)]
        ));
        assert_eq!(cell.truncation, Some(Trunc::Middle));
    }

    #[cfg(feature = "style")]
    #[test]
    fn accepts_colorize_values_for_cells_and_headers() {
        let mut table = Table::with_columns(vec![
            Column::new("Status").bright_green().bold(),
            Column::new("Notes"),
        ]);

        table.column("Status").width(10);
        table.add_row(vec![
            Cell::new("DefinitelyActive").bright_red().underline(),
            Cell::new("Ready"),
        ]);

        let plain = plain_lines(&table);

        assert!(plain[1].contains("Status"));
        assert!(plain[3].contains("Definitel…"));
    }

    #[test]
    fn renders_multiline_headers_and_rows() {
        let mut table = Table::with_columns(vec![Column::new("Name\nAlias"), Column::new("Value")]);
        table.add_row(vec![Cell::new("alpha\nbeta"), Cell::new("1")]);

        assert_eq!(
            plain_lines(&table),
            vec![
                "┌───────┬───────┐",
                "│ Name  │ Value │",
                "│ Alias │       │",
                "├───────┼───────┤",
                "│ alpha │ 1     │",
                "│ beta  │       │",
                "└───────┴───────┘",
            ]
        );
    }

    #[test]
    fn renders_center_aligned_sections_inside_a_single_table() {
        assert_eq!(
            section_table_lines(Align::Center),
            expected_section_lines("├─── Alpha ────┤")
        );
    }

    #[test]
    fn renders_left_aligned_sections_inside_a_single_table() {
        assert_eq!(
            section_table_lines(Align::Left),
            expected_section_lines("├─ Alpha ──────┤")
        );
    }

    #[test]
    fn renders_right_aligned_sections_inside_a_single_table() {
        assert_eq!(
            section_table_lines(Align::Right),
            expected_section_lines("├────── Alpha ─┤")
        );
    }

    #[test]
    fn renders_mid_joints_when_a_section_label_leaves_room() {
        let mut table =
            Table::with_columns(vec![Column::new("A"), Column::new("B"), Column::new("C")]);
        table.add_section("X");
        table.add_row(vec![Cell::new("1"), Cell::new("2"), Cell::new("3")]);

        assert_eq!(
            plain_lines(&table),
            vec![
                "┌───┬───┬───┐",
                "│ A │ B │ C │",
                "├───┼ X ┼───┤",
                "│ 1 │ 2 │ 3 │",
                "└───┴───┴───┘",
            ]
        );
    }

    #[test]
    fn sections_and_separators_can_use_their_own_styles() {
        let table_style = TableStyle {
            top_left: "╔",
            top_right: "╗",
            bottom_left: "╚",
            bottom_right: "╝",
            horiz: "═",
            vert: "║",
            top_joint: "╦",
            mid_left: "╠",
            mid_right: "╣",
            mid_joint: "╬",
            bottom_joint: "╩",
        };
        let section_style = SectionStyle::unicode();
        let separator_style = SectionStyle {
            horiz: "-",
            mid_left: "-",
            mid_right: "-",
            mid_joint: "-",
        };

        let mut table = Table::with_columns(vec![Column::new("Name"), Column::new("Value")])
            .with_style(table_style);

        table.add_section("Alpha").style(section_style);
        table.add_row(vec![Cell::new("a"), Cell::new("1")]);
        table.add_separator().style(separator_style);
        table.add_row(vec![Cell::new("b"), Cell::new("2")]);

        let plain = plain_lines(&table);

        assert!(plain[0].starts_with("╔"));
        assert!(plain[0].ends_with("╗"));
        assert_eq!(plain[2], "├─── Alpha ────┤");
        assert_eq!(plain[4], "----------------");
        assert!(plain[6].starts_with("╚"));
        assert!(plain[6].ends_with("╝"));
    }

    #[test]
    fn applies_column_and_cell_truncation() {
        let mut table = Table::with_columns(vec![Column::new("Value"), Column::new("Other")]);
        table.column("Value").max_width(5).truncate(Trunc::Start);
        table.add_row(vec![Cell::new("abcdefghij"), Cell::new("z")]);
        table.add_row(vec![
            Cell::new("abcdefghij").truncate(Trunc::Middle),
            Cell::new("z"),
        ]);

        assert_eq!(
            plain_lines(&table),
            vec![
                "┌───────┬───────┐",
                "│ Value │ Other │",
                "├───────┼───────┤",
                "│ …ghij │ z     │",
                "│ ab…ij │ z     │",
                "└───────┴───────┘",
            ]
        );
    }

    #[cfg(feature = "style")]
    #[test]
    fn truncation_keeps_ellipsis_tight_and_colored() {
        let mut table = Table::with_columns(vec![Column::new("Name")]);
        table.column(0).max_width(14);
        table.add_row(vec![Cell::new("Cynthia \"CJ\" Lee").bright_red()]);

        let rendered = table.render_lines_for_test(Some(40)).join("\n");
        let plain = strip_ansi(&rendered);

        assert!(plain.contains("Cynthia \"CJ\"…"));
        assert!(!plain.contains("Cynthia \"CJ\" …"));
        assert!(rendered.contains("\x1b[91mCynthia \"CJ\"…\x1b[0m"));
    }

    #[test]
    fn builds_columns_in_one_step() {
        let mut table = Table::with_columns(vec![
            Column::new("Name").width(0.3),
            Column::new("Age").width(0.15),
            Column::new("City").width(0.55),
        ]);

        table.add_row(vec![
            Cell::new("Alice"),
            Cell::new("30"),
            Cell::new("New York"),
        ]);

        let plain = table
            .render_lines_for_test(Some(40))
            .into_iter()
            .map(|line| strip_ansi(&line))
            .collect::<Vec<_>>();

        assert_eq!(plain[0].chars().count(), 40);
        assert!(plain[1].contains("Name"));
        assert!(plain[1].contains("Age"));
        assert!(plain[1].contains("City"));
        assert!(plain[3].contains("Alice"));
    }

    #[test]
    fn renders_fractional_columns_against_terminal_width() {
        let mut table = Table::with_columns(vec![Column::new("Name"), Column::new("Value")]);
        table.column("Name").max_width(0.5);
        table.column("Value").max_width(0.5);
        table.add_row(vec![Cell::new("Alice"), Cell::new("123")]);

        let lines = table.render_lines_for_test(Some(40));
        let plain = lines
            .iter()
            .map(|line| strip_ansi(line))
            .collect::<Vec<_>>();

        assert_eq!(plain[0].chars().count(), 40);
        assert_eq!(plain.last().unwrap().chars().count(), 40);
        assert!(plain[1].contains("Name"));
        assert!(plain[3].contains("Alice"));
    }

    #[test]
    fn newline_truncation_wraps_at_spaces_and_hard_breaks_when_needed() {
        let mut table = Table::with_columns(vec![Column::new("Value")]);
        table.column(0).max_width(8);
        table.add_row(vec![Cell::new("one two three").truncate(Trunc::NewLine)]);
        table.add_row(vec![Cell::new("abcdefghij").truncate(Trunc::NewLine)]);

        assert_eq!(
            plain_lines(&table),
            vec![
                "┌──────────┐",
                "│ Value    │",
                "├──────────┤",
                "│ one two  │",
                "│ three    │",
                "│ abcdefgh │",
                "│ ij       │",
                "└──────────┘",
            ]
        );
    }

    fn plain_lines(table: &Table) -> Vec<String> {
        table
            .render_lines()
            .into_iter()
            .map(|line| strip_ansi(&line))
            .collect()
    }

    fn section_table_lines(align: Align) -> Vec<String> {
        let mut table = Table::with_columns(vec![Column::new("Name"), Column::new("Value")]);
        table.add_section("Alpha").align(align);
        table.add_row(vec![Cell::new("a"), Cell::new("1")]);

        plain_lines(&table)
    }

    fn expected_section_lines(section_line: &str) -> Vec<String> {
        vec![
            "┌──────┬───────┐".to_string(),
            "│ Name │ Value │".to_string(),
            section_line.to_string(),
            "│ a    │ 1     │".to_string(),
            "└──────┴───────┘".to_string(),
        ]
    }
}
