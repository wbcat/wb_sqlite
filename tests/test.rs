//! wb_sqlite derive macro test / example

#![allow(unused)]

use wb_sqlite::{
	CreateIndexSql, CreateTableLogSql, CreateTableSql, Get, Insert, InsertSync, SelectAsSql,
	SelectSql, Update, UpdateSync,
};

#[derive(Debug, Default, CreateTableSql, SelectSql, Insert, InsertSync)]
struct Single {
	id: i64,
}

#[derive(Debug, Default, CreateTableSql, SelectSql, Insert, InsertSync, Get, sqlx::FromRow)]
struct SinglePk {
	#[sql(constraint = "PRIMARY KEY")]
	id: i64,
}

#[derive(
	Debug,
	Default,
	CreateTableSql,
	SelectSql,
	Insert,
	InsertSync,
	Get,
	sqlx::FromRow,
	Update,
	UpdateSync,
)]
struct Double {
	#[sql(constraint = "PRIMARY KEY")]
	id: i64,
	num: u32,
}

#[derive(
	Debug,
	Default,
	CreateTableSql,
	CreateIndexSql,
	CreateTableLogSql,
	SelectSql,
	Get,
	Insert,
	InsertSync,
	Update,
	UpdateSync,
	sqlx::FromRow,
)]
struct Record {
	#[sql(constraint = "PRIMARY KEY")]
	id: i64,
	#[sql(constraint = "REFERENCES single_pk(id)")]
	pub fk: i64,
	#[sql(constraint = "UNIQUE")]
	name: String,
	#[sql(constraint = "CHECK (ok IN (0,1))")]
	ok: bool,
	#[sql(constraint = "CHECK (pos >= 0)")]
	pos: u32,
	num: i64,
	sci_val: f64,
	note: String,
	data: Vec<u8>,
	opt_ok: Option<bool>,
	opt_pos: Option<u32>,
	opt_num: Option<i64>,
	opt_sci_val: Option<f64>,
	opt_note: Option<String>,
	opt_data: Option<Vec<u8>>,
	#[sql(typ = "ANY")]
	any_data: Option<Vec<u8>>,
}

#[derive(
	Debug,
	Default,
	CreateTableSql,
	SelectSql,
	SelectAsSql,
	Get,
	Insert,
	InsertSync,
	Update,
	UpdateSync,
	sqlx::FromRow,
)]
#[sqlas(from = "record where id < 3 order by id desc")]
struct MapRecord {
	#[sql(constraint = "PRIMARY KEY")]
	#[sqlas(col = "0")]
	id: i64,
	name: String,
	#[sqlas(col = "num * sci_val")]
	val: f64,
	#[sqlas(col = "concat(name,'; note: ',note)")]
	note: String,
}

#[derive(
	Debug, Default, CreateTableSql, CreateIndexSql, SelectSql, Insert, InsertSync, sqlx::FromRow,
)]
struct NtoMrel {
	#[sql(constraint = "REFERENCES record(id) ON UPDATE RESTRICT ON DELETE RESTRICT")]
	n: i64,
	#[sql(constraint = "REFERENCES map_record(id) ON UPDATE RESTRICT ON DELETE RESTRICT")]
	m: i64,
}

#[test]
fn create() -> Result<(), rusqlite::Error> {
	fn eq(l: &str, r: &str) {
		assert_eq!(l, r)
	}

	eq(
		Single::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS single (id INTEGER NOT NULL) STRICT;",
	);
	eq(Single::SELECT_SQL, "SELECT id FROM single");
	eq(
		SinglePk::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS single_pk (id INTEGER NOT NULL PRIMARY KEY) STRICT;",
	);
	eq(SinglePk::SELECT_SQL, "SELECT id FROM single_pk");
	eq(
		Double::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS double (id INTEGER NOT NULL PRIMARY KEY, num INTEGER NOT NULL) STRICT;"
	);
	eq(Double::SELECT_SQL, "SELECT id,num FROM double");
	eq(
		Record::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS record (id INTEGER NOT NULL PRIMARY KEY, fk INTEGER NOT NULL REFERENCES single_pk(id), name TEXT NOT NULL UNIQUE, ok INTEGER NOT NULL CHECK (ok IN (0,1)), pos INTEGER NOT NULL CHECK (pos >= 0), num INTEGER NOT NULL, sci_val REAL NOT NULL, note TEXT NOT NULL, data BLOB NOT NULL, opt_ok INTEGER, opt_pos INTEGER, opt_num INTEGER, opt_sci_val REAL, opt_note TEXT, opt_data BLOB, any_data ANY) STRICT;"
	);
	eq(
		Record::CREATE_INDEX_SQL,
		"CREATE INDEX IF NOT EXISTS record_fk_idx ON record(fk); ",
	);
	eq(
		Record::CREATE_TABLE_LOG_SQL,
		"CREATE TABLE IF NOT EXISTS record_log (id INTEGER NOT NULL, fk INTEGER NOT NULL, name TEXT NOT NULL, ok INTEGER NOT NULL, pos INTEGER NOT NULL, num INTEGER NOT NULL, sci_val REAL NOT NULL, note TEXT NOT NULL, data BLOB NOT NULL, opt_ok INTEGER, opt_pos INTEGER, opt_num INTEGER, opt_sci_val REAL, opt_note TEXT, opt_data BLOB, any_data ANY) STRICT; CREATE INDEX IF NOT EXISTS record_log_id_idx ON record_log(id); CREATE TRIGGER IF NOT EXISTS record_update UPDATE ON record BEGIN INSERT INTO record_log (id,fk,name,ok,pos,num,sci_val,note,data,opt_ok,opt_pos,opt_num,opt_sci_val,opt_note,opt_data,any_data) VALUES (OLD.id,OLD.fk,OLD.name,OLD.ok,OLD.pos,OLD.num,OLD.sci_val,OLD.note,OLD.data,OLD.opt_ok,OLD.opt_pos,OLD.opt_num,OLD.opt_sci_val,OLD.opt_note,OLD.opt_data,OLD.any_data); END; CREATE TRIGGER IF NOT EXISTS record_delete DELETE ON record BEGIN INSERT INTO record_log (id,fk,name,ok,pos,num,sci_val,note,data,opt_ok,opt_pos,opt_num,opt_sci_val,opt_note,opt_data,any_data) VALUES (OLD.id,OLD.fk,OLD.name,OLD.ok,OLD.pos,OLD.num,OLD.sci_val,OLD.note,OLD.data,OLD.opt_ok,OLD.opt_pos,OLD.opt_num,OLD.opt_sci_val,OLD.opt_note,OLD.opt_data,OLD.any_data); END;"
	);
	eq(
		Record::SELECT_SQL,
		"SELECT id,fk,name,ok,pos,num,sci_val,note,data,opt_ok,opt_pos,opt_num,opt_sci_val,opt_note,opt_data,any_data FROM record"
	);
	eq(
		MapRecord::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS map_record (id INTEGER NOT NULL PRIMARY KEY, name TEXT NOT NULL, val REAL NOT NULL, note TEXT NOT NULL) STRICT;"
	);
	eq(
		MapRecord::SELECT_SQL,
		"SELECT id,name,val,note FROM map_record",
	);
	eq(
		MapRecord::SELECT_AS_SQL,
		"SELECT 0 AS id,name,num * sci_val AS val,concat(name,'; note: ',note) AS note FROM record where id < 3 order by id desc"
	);
	eq(
		NtoMrel::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS nto_mrel (n INTEGER NOT NULL REFERENCES record(id) ON UPDATE RESTRICT ON DELETE RESTRICT, m INTEGER NOT NULL REFERENCES map_record(id) ON UPDATE RESTRICT ON DELETE RESTRICT) STRICT;"
	);
	eq(
		NtoMrel::CREATE_INDEX_SQL,
		"CREATE INDEX IF NOT EXISTS nto_mrel_n_idx ON nto_mrel(n); CREATE INDEX IF NOT EXISTS nto_mrel_m_idx ON nto_mrel(m); "
	);
	eq(NtoMrel::SELECT_SQL, "SELECT n,m FROM nto_mrel");

	let c = rusqlite::Connection::open_in_memory()?;
	fn x(c: &rusqlite::Connection, sql: &str) {
		match c.execute_batch(sql) {
			Ok(_) => (),
			Err(err) => panic!("{err}"),
		};
	}
	x(&c, Single::CREATE_TABLE_SQL);
	x(&c, SinglePk::CREATE_TABLE_SQL);
	x(&c, Double::CREATE_TABLE_SQL);
	x(&c, Record::CREATE_TABLE_SQL);
	x(&c, Record::CREATE_INDEX_SQL);
	x(&c, Record::CREATE_TABLE_LOG_SQL);
	x(&c, MapRecord::CREATE_TABLE_SQL);
	x(&c, NtoMrel::CREATE_TABLE_SQL);
	x(&c, NtoMrel::CREATE_INDEX_SQL);

	Ok(())
}

#[test]
fn rusqlite() -> Result<(), rusqlite::Error> {
	let c = rusqlite::Connection::open_in_memory()?;
	c.execute_batch(Single::CREATE_TABLE_SQL)?;
	c.execute_batch(SinglePk::CREATE_TABLE_SQL)?;
	c.execute_batch(Double::CREATE_TABLE_SQL)?;
	c.execute_batch(Record::CREATE_TABLE_SQL)?;
	c.execute_batch(Record::CREATE_INDEX_SQL)?;
	c.execute_batch(Record::CREATE_TABLE_LOG_SQL)?;
	c.execute_batch(MapRecord::CREATE_TABLE_SQL)?;
	c.execute_batch(NtoMrel::CREATE_TABLE_SQL)?;
	c.execute_batch(NtoMrel::CREATE_INDEX_SQL)?;

	let single = Single::default();
	let single_id = single.insert_sync(&c)?;

	let single_pk = SinglePk::default();
	let single_pk_id = single_pk.insert_sync(&c)?;

	let mut r = Record::default();
	r.fk = single_pk_id;
	r.name = "me".to_owned();
	let id = r.insert_sync(&c).unwrap();
	assert!(id == 1);
	r.id = id;
	r.name = "you".to_owned();
	let ok = r.update_sync(&c)?;
	assert!(ok);

	Ok(())
}

#[tokio::test]
async fn sqlx() -> Result<(), sqlx::Error> {
	use sqlx::{Connection, Executor, SqliteConnection};
	let mut c = SqliteConnection::connect(":memory:").await?;
	c.execute(Single::CREATE_TABLE_SQL).await?;
	c.execute(SinglePk::CREATE_TABLE_SQL).await?;
	c.execute(Double::CREATE_TABLE_SQL).await?;
	c.execute(Record::CREATE_TABLE_SQL).await?;
	c.execute(Record::CREATE_INDEX_SQL).await?;
	c.execute(Record::CREATE_TABLE_LOG_SQL).await?;
	c.execute(MapRecord::CREATE_TABLE_SQL).await?;
	c.execute(NtoMrel::CREATE_TABLE_SQL).await?;
	c.execute(NtoMrel::CREATE_INDEX_SQL).await?;

	let single_pk = SinglePk::default();
	let single_pk_id = single_pk.insert(&mut c).await?;

	let mut r = Record::default();
	r.fk = single_pk_id;
	r.name = "me".to_owned();
	let id = r.insert(&mut c).await?;
	assert!(id == 1);
	r.id = id;
	r.name = "you".to_owned();
	let ok = r.update(&mut c).await?;
	assert!(ok);

	Ok(())
}
