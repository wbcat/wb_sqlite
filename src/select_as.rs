use virtue::{
	parse::Attribute,
	prelude::{AttributeAccess, Body, Fields, Generator, Parse, Result, TokenStream},
};

pub(crate) fn inner(input: TokenStream) -> Result<TokenStream> {
	let parse = Parse::new(input)?;
	let (mut generator, attributes, body) = parse.into_generator();
	match body {
		Body::Struct(struct_body) => gen_struct(&mut generator, attributes, struct_body.fields)?,
		Body::Enum(_enum_body) => unimplemented!(),
	};
	generator.export_to_file("wb_sqlite", "SelectAsSql");
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
		.get_attribute::<crate::util::AsTabAttr>()?
		.unwrap_or_default();
	let from = if tab_attr.from.is_empty() {
		tab_name
	} else {
		tab_attr.from
	};

	let mut columns = String::new();
	for (ident, uf) in struct_fields {
		let col_attr = uf
			.attributes
			.get_attribute::<crate::util::AsColAttr>()?
			.unwrap_or_default();
		if col_attr.col.is_empty() {
			columns.push_str(&ident.to_string());
		} else {
			columns.push_str(&format!("{} AS {}", col_attr.col, ident));
		};
		columns.push(',');
	}
	columns.pop(); // get rid of the last ','

	generator
		.generate_impl()
		.generate_const("SELECT_AS_SQL", "&'static str")
		.make_pub()
		.with_value(|b| {
			b.push_parsed(format!("\"SELECT {columns} FROM {from}\""))?;
			Ok(())
		})?;
	Ok(())
}
