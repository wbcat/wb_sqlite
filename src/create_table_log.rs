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
	generator.export_to_file("wb_sqlite", "CreateTableLogSql");
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
	let tab_log_name = format!("{tab_name}_log");

	let mut col_defs = String::new();
	let mut columns = String::new();
	let mut log_values = String::new();
	let mut create_index = String::new();
	for (ident, uf) in struct_fields {
		let col_attr = uf
			.attributes
			.get_attribute::<crate::util::ColAttr>()?
			.unwrap_or_default();
		if !col_defs.is_empty() {
			col_defs.push_str(", ");
			columns.push(',');
			log_values.push(',');
		}
		let col_name = ident.to_string();
		col_defs.push_str(&col_name);
		columns.push_str(&col_name);
		log_values.push_str("OLD.");
		log_values.push_str(&col_name);
		col_defs.push(' ');
		if col_attr.typ.is_empty() {
			col_defs.push_str(crate::col_typ(&uf.type_string()));
		} else {
			col_defs.push_str(&col_attr.typ);
		}
		if col_attr.constraint.starts_with("PRIMARY KEY") {
			create_index = format!(
				"CREATE INDEX IF NOT EXISTS {tab_log_name}_{col_name}_idx ON {tab_log_name}({col_name});"
			);
		}
	}

	generator
		.generate_impl()
		.generate_const("CREATE_TABLE_LOG_SQL", "&'static str")
		.make_pub()
		.with_value(|b| {
			b.push_parsed(format!(
				"\"CREATE TABLE IF NOT EXISTS {tab_log_name} ({col_defs}) STRICT; {create_index} CREATE TRIGGER IF NOT EXISTS {tab_name}_update UPDATE ON {tab_name} BEGIN INSERT INTO {tab_log_name} ({columns}) VALUES ({log_values}); END; CREATE TRIGGER IF NOT EXISTS {tab_name}_delete DELETE ON {tab_name} BEGIN INSERT INTO {tab_log_name} ({columns}) VALUES ({log_values}); END;\""
			))?;
			Ok(())
		})?;
	Ok(())
}
