use virtue::{
	parse::Attribute,
	prelude::{AttributeAccess, Body, Fields, FnSelfArg, Generator, Parse, Result, TokenStream},
};

pub(crate) fn inner(input: TokenStream) -> Result<TokenStream> {
	let parse = Parse::new(input)?;
	let (mut generator, attributes, body) = parse.into_generator();
	match body {
		Body::Struct(struct_body) => gen_struct(&mut generator, attributes, struct_body.fields)?,
		Body::Enum(_enum_body) => unimplemented!(),
	};
	generator.export_to_file("wb_sqlite", "InsertSync");
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
	let mut columns = String::new();
	let mut columns_full = String::new();
	let mut values = String::new();
	let mut values_full = String::new();
	for (ident, uf) in struct_fields {
		let col_attr = uf
			.attributes
			.get_attribute::<crate::util::ColAttr>()?
			.unwrap_or_default();
		columns_full.push_str(&ident.to_string());
		columns_full.push(',');
		values_full.push_str("?,");
		if col_attr.constraint.starts_with("PRIMARY KEY") {
			pk = ident.to_string();
			pk_typ = uf.type_string();
		} else {
			columns.push_str(&ident.to_string());
			columns.push(',');
			values.push_str("?,");
		}
	}
	// get rid of the last ','
	columns.pop();
	columns_full.pop();
	values.pop();
	values_full.pop();

	fn stmt(tab_name: &str, columns: &str, values: &str) -> String {
		format!("let mut stmt = conn.prepare_cached(\"INSERT INTO {tab_name} ({columns}) VALUES ({values})\")?;")
	}

	fn insert(columns: &str) -> String {
		let mut s = String::new();
		s.push_str("stmt.insert(::rusqlite::params![");
		for c in columns.split(',') {
			s.push_str(&format!("self.{c},"))
		}
		s.pop(); // get rid of the last ','
		s.push_str("])?");
		s
	}

	generator
		.generate_impl()
		.generate_fn("insert_sync")
		.with_self_arg(FnSelfArg::RefSelf)
		.with_arg("conn", "&::rusqlite::Connection")
		.with_return_type("Result<i64, ::rusqlite::Error>")
		.make_pub()
		.body(|fn_body| {
			let mut s = String::new();
			if pk.is_empty() || pk_typ != "i64" {
				s.push_str(&stmt(&tab_name, &columns_full, &values_full));
				s.push_str(&format!("let rowid = {}; Ok(rowid)", insert(&columns_full)));
			} else {
				s.push_str(&format!("let rowid = if self.{pk} > 0 {{"));
				s.push_str(&stmt(&tab_name, &columns_full, &values_full));
				s.push_str(&insert(&columns_full));
				s.push_str(" } else { ");
				s.push_str(&stmt(&tab_name, &columns, &values));
				s.push_str(&insert(&columns));
				s.push_str(" }; Ok(rowid)");
			}
			fn_body.push_parsed(s)?;
			Ok(())
		})?;
	Ok(())
}
