use virtue::{
	parse::Attribute,
	prelude::{Body, Fields, Generator, Parse, Result, TokenStream},
};

pub fn inner(input: TokenStream) -> Result<TokenStream> {
	let parse = Parse::new(input)?;
	let (mut generator, attributes, body) = parse.into_generator();
	match body {
		Body::Struct(struct_body) => gen_struct(&mut generator, attributes, struct_body.fields)?,
		Body::Enum(_enum_body) => unimplemented!(),
	};
	generator.export_to_file("wb_sqlite", "SelectSql");
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

	let mut columns = String::new();
	for (ident, _uf) in struct_fields {
		columns.push_str(&ident.to_string());
		columns.push(',');
	}
	columns.pop(); // get rid of the last ','

	generator
		.generate_impl()
		.generate_const("SELECT_SQL", "&'static str")
		.make_pub()
		.with_value(|b| {
			b.push_parsed(format!("\"SELECT {columns} FROM {tab_name}\""))?;
			Ok(())
		})?;
	Ok(())
}
