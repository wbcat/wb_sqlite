//! wb_sqlite derive macro test / example

#![allow(unused)]

use wb_sqlite::{
	CreateIndexSql, CreateTableLogSql, CreateTableSql, Get, Insert, InsertSync, SelectAsSql,
	SelectSql, Update, UpdateSync,
};

#[derive(
	Debug,
	Default,
	CreateTableSql,
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
	#[sql(constraint = "UNIQUE CHECK(name != '')")]
	name: String,
	ok: bool,
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
	Debug,
	Default,
	CreateTableSql,
	CreateIndexSql,
	SelectSql,
	Get, // no generate
	Insert,
	InsertSync,
	Update,     // no generate
	UpdateSync, // no generate
	sqlx::FromRow,
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
		Record::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS record (id INTEGER NOT NULL PRIMARY KEY, name TEXT NOT NULL UNIQUE CHECK(name != ''), ok INTEGER NOT NULL, pos INTEGER NOT NULL, num INTEGER NOT NULL, sci_val REAL NOT NULL, note TEXT NOT NULL, data BLOB NOT NULL, opt_ok INTEGER, opt_pos INTEGER, opt_num INTEGER, opt_sci_val REAL, opt_note TEXT, opt_data BLOB, any_data ANY) STRICT;"
	);
	eq(
		Record::CREATE_TABLE_LOG_SQL,
		"CREATE TABLE IF NOT EXISTS record_log (id INTEGER NOT NULL, name TEXT NOT NULL, ok INTEGER NOT NULL, pos INTEGER NOT NULL, num INTEGER NOT NULL, sci_val REAL NOT NULL, note TEXT NOT NULL, data BLOB NOT NULL, opt_ok INTEGER, opt_pos INTEGER, opt_num INTEGER, opt_sci_val REAL, opt_note TEXT, opt_data BLOB, any_data ANY) STRICT; CREATE INDEX IF NOT EXISTS record_log_id_idx ON record_log(id); CREATE TRIGGER IF NOT EXISTS record_update UPDATE ON record BEGIN INSERT INTO record_log (id,name,ok,pos,num,sci_val,note,data,opt_ok,opt_pos,opt_num,opt_sci_val,opt_note,opt_data,any_data) VALUES (OLD.id,OLD.name,OLD.ok,OLD.pos,OLD.num,OLD.sci_val,OLD.note,OLD.data,OLD.opt_ok,OLD.opt_pos,OLD.opt_num,OLD.opt_sci_val,OLD.opt_note,OLD.opt_data,OLD.any_data); END; CREATE TRIGGER IF NOT EXISTS record_delete DELETE ON record BEGIN INSERT INTO record_log (id,name,ok,pos,num,sci_val,note,data,opt_ok,opt_pos,opt_num,opt_sci_val,opt_note,opt_data,any_data) VALUES (OLD.id,OLD.name,OLD.ok,OLD.pos,OLD.num,OLD.sci_val,OLD.note,OLD.data,OLD.opt_ok,OLD.opt_pos,OLD.opt_num,OLD.opt_sci_val,OLD.opt_note,OLD.opt_data,OLD.any_data); END;"
	);
	eq(
		MapRecord::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS map_record (id INTEGER NOT NULL PRIMARY KEY, name TEXT NOT NULL, val REAL NOT NULL, note TEXT NOT NULL) STRICT;"
	);
	eq(
		NtoMrel::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS nto_mrel (n INTEGER NOT NULL REFERENCES record(id) ON UPDATE RESTRICT ON DELETE RESTRICT, m INTEGER NOT NULL REFERENCES map_record(id) ON UPDATE RESTRICT ON DELETE RESTRICT) STRICT;"
	);
	eq(
		NtoMrel::CREATE_INDEX_SQL,
		"CREATE INDEX IF NOT EXISTS nto_mrel_n_idx ON nto_mrel(n); CREATE INDEX IF NOT EXISTS nto_mrel_m_idx ON nto_mrel(m); "
	);

	let c = rusqlite::Connection::open_in_memory()?;
	fn x(c: &rusqlite::Connection, sql: &str) {
		match c.execute_batch(sql) {
			Ok(_) => (),
			Err(err) => panic!("{err}"),
		};
	}
	x(&c, Record::CREATE_TABLE_SQL);
	x(&c, Record::CREATE_TABLE_LOG_SQL);
	x(&c, MapRecord::CREATE_TABLE_SQL);
	x(&c, NtoMrel::CREATE_TABLE_SQL);
	x(&c, NtoMrel::CREATE_INDEX_SQL);

	Ok(())
}

#[test]
fn rusqlite() -> Result<(), rusqlite::Error> {
	let c = rusqlite::Connection::open_in_memory()?;
	c.execute_batch(Record::CREATE_TABLE_SQL)?;
	c.execute_batch(Record::CREATE_TABLE_LOG_SQL)?;
	c.execute_batch(MapRecord::CREATE_TABLE_SQL)?;
	c.execute_batch(NtoMrel::CREATE_TABLE_SQL)?;
	c.execute_batch(NtoMrel::CREATE_INDEX_SQL)?;

	let mut r = Record::default();

	r.name = "me".to_owned();
	let id = r.insert_sync(&c)?;
	assert!(id == 1);

	r.id = id;
	r.name = "you".to_owned();
	let ok = r.update_sync(&c)?;
	assert!(ok);

	// ToDo: MapRecord select as, TableLog auslesen

	Ok(())
}

#[tokio::test]
async fn sqlx() -> Result<(), sqlx::Error> {
	use sqlx::{Connection, Executor, SqliteConnection};
	let mut c = SqliteConnection::connect(":memory:").await?;
	c.execute(Record::CREATE_TABLE_SQL).await?;
	c.execute(Record::CREATE_TABLE_LOG_SQL).await?;
	c.execute(MapRecord::CREATE_TABLE_SQL).await?;
	c.execute(NtoMrel::CREATE_TABLE_SQL).await?;
	c.execute(NtoMrel::CREATE_INDEX_SQL).await?;

	let mut r = Record::default();

	r.name = "me".to_owned();
	let id = r.insert(&mut c).await?;
	assert!(id == 1);

	r.id = id;
	r.name = "you".to_owned();
	let ok = r.update(&mut c).await?;
	assert!(ok);

	Ok(())
}
