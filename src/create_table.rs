use virtue::{
	parse::Attribute,
	prelude::{AttributeAccess, Body, Fields, Generator, Parse, Result, TokenStream},
};

pub fn inner(input: TokenStream) -> Result<TokenStream> {
	let parse = Parse::new(input)?;
	let (mut generator, attributes, body) = parse.into_generator();
	match body {
		Body::Struct(struct_body) => gen_struct(&mut generator, attributes, struct_body.fields)?,
		Body::Enum(_enum_body) => unimplemented!(),
	};
	generator.export_to_file("wb_sqlite", "CreateTableSql");
	generator.finish()
}

fn gen_struct(
	generator: &mut Generator,
	attributes: Vec<Attribute>,
	fields: Option<Fields>,
) -> Result {
	let Some(Fields::Struct(struct_fields)) = fields else {
		return Ok(());
	};
	let tab_name = crate::util::tab_name(&generator.target_name().to_string());
	let tab_attr = attributes
		.get_attribute::<crate::util::TabAttr>()?
		.unwrap_or_default();
	let tab_constraint = if tab_attr.constraint.is_empty() {
		String::new()
	} else {
		format!(", {}", &tab_attr.constraint)
	};
	let tab_option = if tab_attr.option.is_empty() {
		String::new()
	} else {
		format!(", {}", &tab_attr.option)
	};

	let mut col_defs = String::new();
	for (ident, uf) in struct_fields {
		let col_attr = uf
			.attributes
			.get_attribute::<crate::util::ColAttr>()?
			.unwrap_or_default();
		if !col_defs.is_empty() {
			col_defs.push_str(", ");
		}
		col_defs.push_str(&ident.to_string());
		col_defs.push(' ');
		if col_attr.typ.is_empty() {
			col_defs.push_str(crate::util::col_typ(&uf.type_string()));
		} else {
			col_defs.push_str(&col_attr.typ);
		}
		if !col_attr.constraint.is_empty() {
			col_defs.push(' ');
			col_defs.push_str(&col_attr.constraint);
		}
	}

	generator
		.generate_impl()
		.generate_const("CREATE_TABLE_SQL", "&'static str")
		.make_pub()
		.with_value(|b| {
			b.push_parsed(format!(
				"\"CREATE TABLE IF NOT EXISTS {tab_name} ({col_defs}{tab_constraint}) STRICT{tab_option};\""
			))?;
			Ok(())
		})?;
	Ok(())
}
