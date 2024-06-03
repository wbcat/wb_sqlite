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
	generator.export_to_file("wb_sqlite", "CreateIndexSql");
	generator.finish()
}

fn gen_struct(
	generator: &mut Generator,
	_attributes: Vec<Attribute>,
	fields: Option<Fields>,
) -> Result {
	let Some(Fields::Struct(struct_fields)) = fields else {
		return Ok(());
	};
	let tab_name = crate::util::tab_name(&generator.target_name().to_string());

	let mut create_index = String::new();
	for (ident, uf) in struct_fields {
		let col_attr = uf
			.attributes
			.get_attribute::<crate::util::ColAttr>()?
			.unwrap_or_default();
		if col_attr.constraint.starts_with("REFERENCES ") {
			let col_name = ident.to_string();
			create_index.push_str(&format!(
				"CREATE INDEX IF NOT EXISTS {tab_name}_{col_name}_idx ON {tab_name}({col_name});"
			));
		}
	}

	generator
		.generate_impl()
		.generate_const("CREATE_INDEX_SQL", "&'static str")
		.make_pub()
		.with_value(|b| {
			b.push_parsed(format!("\"{create_index}\""))?;
			Ok(())
		})?;
	Ok(())
}
