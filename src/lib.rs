//! A terminal table renderer with column sizing, truncation, wrapping, and
//! section separators.
//!
//! The crate is designed around three core building blocks:
//!
//! - [`Table`] holds headers, rows, and the rendering configuration.
//! - [`Column`] defines a header plus default styling for a column.
//! - [`Cell`] lets you override styling or truncation on individual values.
//!
//! # Quick Start
//!
//! ```rust
//! use tiny_table::{Cell, Column, Align, Table, Trunc};
//!
//! let mut table = Table::with_columns(vec![
//!     Column::new("Name").width(0.30),
//!     Column::new("Role").truncate(Trunc::Middle),
//!     Column::new("Status"),
//! ]);
//!
//! table.add_section("Team").align(Align::Left);
//! table.add_row(vec![
//!     Cell::new("Ada Lovelace"),
//!     Cell::new("Principal Engineer"),
//!     Cell::new("Active").bright_green().bold(),
//! ]);
//!
//! let rendered = table.render();
//! assert!(rendered.contains("Name"));
//! assert!(rendered.contains("Ada Lovelace"));
//! ```
//!
//! # How It Fits Together
//!
//! - Use [`Table::with_columns`] when you already know the schema.
//! - Use [`Table::add_row`] to append values.
//! - Use [`Table::column`] to tweak one column by index or header text.
//! - Use [`Table::add_section`] or [`Table::add_separator`] to break the table
//!   into visual groups.
//! - Use [`Table::render`] when you want the formatted string, or [`Table::print`]
//!   when you just want to write it to standard output.
//!
//! When a terminal width is available, fractional columns are distributed across
//! the remaining content width after fixed and content-based columns are resolved.

#![warn(missing_docs)]

/// Color types and ANSI escape-code generation used by the styling API.
#[cfg(feature = "style")]
pub mod color;
pub mod table;

/// A convenience type for specifying truecolor values by RGB components.
#[cfg(feature = "style")]
pub use color::CustomColor;
/// Alignment options for section labels.
pub use table::Align;
/// A table cell containing content and optional styling overrides.
pub use table::Cell;
/// A color selector used by styling methods.
#[cfg(feature = "style")]
pub use table::Color;
/// A column definition with a header and default styling.
pub use table::Column;
/// A selector used to style a column by index or header text.
pub use table::ColumnTarget;
/// A column width configuration used when rendering a table.
pub use table::ColumnWidth;
/// A section definition used to group rows and render a label.
pub use table::SectionStyle;
/// The main table type.
pub use table::Table;
/// A border style used for tables, sections, and separators.
pub use table::TableStyle;
/// Truncation modes for cell content.
pub use table::Trunc;
#[cfg(feature = "style")]
pub use table::style::{StyleAction, apply_style_actions};
