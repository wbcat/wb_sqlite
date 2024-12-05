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
	generator.export_to_file("wb_sqlite", "Get");
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

	let mut pk = String::new();
	let mut pk_typ = String::new();
	let mut unique = String::new();
	let mut unique_typ = String::new();
	let mut columns = String::new();
	for (ident, uf) in struct_fields {
		let col_attr = uf
			.attributes
			.get_attribute::<crate::util::ColAttr>()?
			.unwrap_or_default();
		if col_attr.constraint.starts_with("PRIMARY KEY") {
			pk = ident.to_string();
			pk_typ = uf.type_string();
			if pk_typ == "String" {
				"&str".clone_into(&mut pk_typ)
			}
		} else if col_attr.constraint.starts_with("UNIQUE") {
			unique.push_str(&ident.to_string());
			unique.push(',');
			let typ = uf.type_string();
			if typ == "String" {
				unique_typ.push_str("&str")
			} else {
				unique_typ.push_str(&typ)
			}
			unique_typ.push(',');
		}
		columns.push_str(&ident.to_string());
		columns.push(',');
	}
	columns.pop(); // get rid of the last ','

	if !pk.is_empty() || !unique.is_empty() {
		let mut gen_impl = generator.generate_impl();
		if !pk.is_empty() {
			gen_impl
			.generate_fn(format!("get_by_{pk}"))
			.as_async()
			.with_arg(&pk, &pk_typ)
			.with_arg("exec", "impl ::sqlx::SqliteExecutor<'_>")
			.with_return_type("Result<Self, ::sqlx::Error>")
			.make_pub()
			.body(|fn_body| {
				let mut s = String::new();
				if pk_typ == "i64" {
					s.push_str(&format!("if {pk} < 1 {{Err(sqlx::Error::RowNotFound)}} else {{"));
				}
				s.push_str(&format!("::sqlx::query_as::<_, Self>(\"SELECT {columns} FROM {tab_name} WHERE {pk}=?\").bind({pk}).fetch_one(exec).await"));
				if pk_typ == "i64" {
					s.push('}');
				}
				fn_body.push_parsed(s)?;
				Ok(())
			})?;
		}
		if !unique.is_empty() {
			unique.pop();
			unique_typ.pop();
			for (col, typ) in std::iter::zip(unique.split(','), unique_typ.split(',')) {
				gen_impl
			.generate_fn(format!("get_by_{col}"))
			.as_async()
			.with_arg(col, typ)
			.with_arg("exec", "impl ::sqlx::SqliteExecutor<'_>")
			.with_return_type("Result<Self, ::sqlx::Error>")
			.make_pub()
			.body(|fn_body| {
				let s = format!("::sqlx::query_as::<_, Self>(\"SELECT {columns} FROM {tab_name} WHERE {col}=?\").bind({col}).fetch_one(exec).await");
				fn_body.push_parsed(s)?;
				Ok(())
			})?;
			}
		}
	}
	Ok(())
}
