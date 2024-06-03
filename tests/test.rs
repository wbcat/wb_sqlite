#![allow(unused)]

#[test]
fn rusqlite() -> Result<(), rusqlite::Error> {
	use wb_sqlite::CreateTableSql;

	fn c(sql: &str) {
		let c = rusqlite::Connection::open_in_memory().unwrap();
		match c.execute_batch(sql) {
			Ok(_) => (),
			Err(err) => panic!("{err}"),
		};
	}

	fn eq(sql: &str, cmp: &str) {
		assert_eq!(sql, cmp);
		c(sql);
	}

	#[derive(CreateTableSql)]
	#[sql(option = "WITHOUT ROWID")]
	pub struct Person {
		#[sql(constraint = "PRIMARY KEY")]
		pub id: i64,
		#[sql(constraint = "UNIQUE CHECK (name != '')")]
		pub name: String, // email
		#[sql(constraint = "DEFAULT ''")]
		pub last_login: String,
		#[sql(constraint = "DEFAULT ''")]
		pub pw_hash: String,
		#[sql(constraint = "DEFAULT NULL")]
		pub roles: Option<String>,
		#[sql(typ = "ANY", constraint = "DEFAULT NULL")]
		pub timestamp: String,
	}
	eq(
		Person::CREATE_TABLE_SQL,
		"CREATE TABLE IF NOT EXISTS person (id INTEGER NOT NULL PRIMARY KEY, name TEXT NOT NULL UNIQUE CHECK (name != ''), last_login TEXT NOT NULL DEFAULT '', pw_hash TEXT NOT NULL DEFAULT '', roles TEXT DEFAULT NULL, timestamp ANY DEFAULT NULL) STRICT, WITHOUT ROWID;"
	);

	Ok(())
}

#[tokio::test]
async fn sqlx() -> Result<(), sqlx::Error> {
	use wb_sqlite::{CreateTableSql, Insert, Update};

	#[derive(CreateTableSql, Insert, Update)]
	struct Person {
		#[sql(constraint = "PRIMARY KEY")]
		id: i64,
		#[sql(constraint = "UNIQUE")]
		name: String,
	}

	use sqlx::{Connection, Executor, SqliteConnection};

	let mut conn = SqliteConnection::connect(":memory:").await?;

	conn.execute(Person::CREATE_TABLE_SQL).await?;

	let p = Person {
		id: 0,
		name: "me".to_owned(),
	};
	let id = p.insert(&mut conn).await?;
	assert!(id > 0);

	let p2 = Person {
		id,
		name: "you".to_owned(),
	};
	let ok = p2.update(&mut conn).await?;
	assert!(ok);

	Ok(())
}
