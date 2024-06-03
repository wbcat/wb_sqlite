use virtue::{
	parse::Attribute,
	prelude::{AttributeAccess, Body, Fields, FnSelfArg, Generator, Parse, Result, TokenStream},
};

pub fn inner(input: TokenStream) -> Result<TokenStream> {
	let parse = Parse::new(input)?;
	let (mut generator, attributes, body) = parse.into_generator();
	match body {
		Body::Struct(struct_body) => gen_struct(&mut generator, attributes, struct_body.fields)?,
		Body::Enum(_enum_body) => unimplemented!(),
	};
	generator.export_to_file("wb_sqlite", "UpdateSync");
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
	let mut columns = String::new();
	for (ident, uf) in struct_fields {
		let col_attr = uf
			.attributes
			.get_attribute::<crate::util::ColAttr>()?
			.unwrap_or_default();
		if col_attr.constraint.starts_with("PRIMARY KEY") {
			pk = ident.to_string();
		} else {
			columns.push_str(&ident.to_string());
			columns.push(',');
		}
	}
	// get rid of the last ','
	columns.pop();

	fn exec(tab_name: &str, pk: &str, columns: &str) -> String {
		let mut s = format!("let mut stmt = conn.prepare_cached(\"UPDATE {tab_name} SET ");
		for c in columns.split(',') {
			s.push_str(&format!("{c}=?,"))
		}
		s.pop();
		s.push_str(&format!(" WHERE {pk}=?\")?;"));

		s.push_str("let rows = stmt.execute(::rusqlite::params![");
		for c in columns.split(',') {
			s.push_str(&format!("self.{c},"))
		}
		s.push_str(&format!("self.{pk}"));
		s.push_str("])?;");
		s
	}

	if !pk.is_empty() {
		generator
			.generate_impl()
			.generate_fn("update_sync")
			.with_self_arg(FnSelfArg::RefSelf)
			.with_arg("conn", "&::rusqlite::Connection")
			.with_return_type("Result<bool, ::rusqlite::Error>")
			.make_pub()
			.body(|fn_body| {
				let mut s = String::new();
				s.push_str(&format!("assert!(self.{pk} > 0);"));
				s.push_str(&exec(&tab_name, &pk, &columns));
				s.push_str("assert!(rows < 2); Ok(rows == 1)");
				fn_body.push_parsed(s)?;
				Ok(())
			})?;
	}
	Ok(())
}
