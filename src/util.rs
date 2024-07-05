use convert_case::{Boundary, Case, Casing};
use virtue::{
	prelude::{Error, FromAttribute, Group, Literal, Result},
	utils::{parse_tagged_attribute, ParsedAttribute},
};

/// Convert TypeName (Pascal) to table_name (Snake)
///
/// <https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md>
pub(crate) fn tab_name(ident: &str) -> String {
	ident
		.from_case(Case::Pascal)
		.without_boundaries(&[
			Boundary::UpperDigit,
			Boundary::DigitLower,
			Boundary::LowerDigit,
		])
		.to_case(Case::Snake)
}

/// Get SQLite column datatype from given rust type-string.
pub(crate) fn col_typ(rust_typ: &str) -> &'static str {
	match rust_typ {
		"bool" | "u8" | "u16" | "u32" | "i8" | "i16" | "i32" | "i64" => "INTEGER NOT NULL",
		"f32" | "f64" => "REAL NOT NULL",
		"&str" | "String" => "TEXT NOT NULL",
		"&[u8]" | "Vec<u8>" => "BLOB NOT NULL",
		"Option<bool>" | "Option<u8>" | "Option<u16>" | "Option<u32>" | "Option<i8>"
		| "Option<i16>" | "Option<i32>" | "Option<i64>" => "INTEGER",
		"Option<f32>" | "Option<f64>" => "REAL",
		"Option<String>" => "TEXT",
		"Option<Vec<u8>>" => "BLOB",
		_ => "ANY",
	}
}

#[derive(Debug, Default)]
pub(crate) struct TabAttr {
	pub(crate) constraint: String, // table-constraint(s)
	pub(crate) option: String,     // table-option other than STRICT
}

impl FromAttribute for TabAttr {
	fn parse(group: &Group) -> Result<Option<Self>> {
		let Some(attributes) = parse_tagged_attribute(group, "sql")? else {
			return Ok(None);
		};
		let mut tab = Self::default();
		for attr in attributes {
			match attr {
				ParsedAttribute::Tag(key) => {
					return Err(Error::custom_at("unknown table attr", key.span()))
				}
				ParsedAttribute::Property(key, val) => match key.to_string().as_str() {
					"constraint" => tab.constraint = literal_str(val)?,
					"option" => tab.option = literal_str(val)?,
					_ => return Err(Error::custom_at("unknown table attr", key.span())),
				},
				_ => {}
			}
		}
		Ok(Some(tab))
	}
}

#[derive(Debug, Default)]
pub(crate) struct ColAttr {
	pub(crate) typ: String,        // type-name
	pub(crate) constraint: String, // column-constraint
}

impl FromAttribute for ColAttr {
	fn parse(group: &Group) -> Result<Option<Self>> {
		let Some(attributes) = parse_tagged_attribute(group, "sql")? else {
			return Ok(None);
		};
		let mut col = Self::default();
		for attr in attributes {
			match attr {
				ParsedAttribute::Tag(i) => {
					return Err(Error::custom_at("unknown column attr", i.span()))
				}
				ParsedAttribute::Property(key, val) => match key.to_string().as_str() {
					"typ" => col.typ = literal_str(val)?,
					"constraint" => col.constraint = literal_str(val)?,
					_ => return Err(Error::custom_at("unknown column attr", key.span())),
				},
				_ => {}
			}
		}
		Ok(Some(col))
	}
}

#[derive(Debug, Default)]
pub(crate) struct AsTabAttr {
	pub(crate) from: String, // SelectAs from value
}

impl FromAttribute for AsTabAttr {
	fn parse(group: &Group) -> Result<Option<Self>> {
		let Some(attributes) = parse_tagged_attribute(group, "sqlas")? else {
			return Ok(None);
		};
		let mut tab = Self::default();
		for attr in attributes {
			match attr {
				ParsedAttribute::Tag(i) => {
					return Err(Error::custom_at("unknown table attr", i.span()))
				}
				ParsedAttribute::Property(key, val) => match key.to_string().as_str() {
					"from" => tab.from = literal_str(val)?,
					_ => return Err(Error::custom_at("unknown table attr", key.span())),
				},
				_ => {}
			}
		}
		Ok(Some(tab))
	}
}

#[derive(Debug, Default)]
pub(crate) struct AsColAttr {
	pub(crate) col: String, // SelectAs original column name
}

impl FromAttribute for AsColAttr {
	fn parse(group: &Group) -> Result<Option<Self>> {
		let Some(attributes) = parse_tagged_attribute(group, "sqlas")? else {
			return Ok(None);
		};
		let mut col = Self::default();
		for attr in attributes {
			match attr {
				ParsedAttribute::Tag(i) => {
					return Err(Error::custom_at("unknown column attr", i.span()))
				}
				ParsedAttribute::Property(key, val) => match key.to_string().as_str() {
					"col" => col.col = literal_str(val)?,
					_ => return Err(Error::custom_at("unknown column attr", key.span())),
				},
				_ => {}
			}
		}
		Ok(Some(col))
	}
}

/// Helper for impl FromAttribute
fn literal_str(val: Literal) -> Result<String> {
	let val_string = val.to_string();
	if val_string.starts_with('"') && val_string.ends_with('"') {
		Ok(val_string[1..val_string.len() - 1].to_string())
	} else {
		Err(Error::custom_at("should be a literal str", val.span()))
	}
}
