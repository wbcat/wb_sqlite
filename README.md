# wb_sqlite
SQLite derive macros.\
Map Rust struct / field to SQLite table / column.\
Generate const + fn (async sqlx / sync rusqlite).

const CREATE table, index, log

const SELECT {fields} FROM {table}

fn insert INSERT INTO {table} ...

fn update UPDATE {table} SET ... WHERE {pk} =

fn get_by_{field-name} for PRIMARY KEY + UNIQUE columns

All derived items are saved to `target/generated/wb_sqlite` thanks to [virtue](https://docs.rs/virtue).

## Examples

### Create table
```rust
# use wb_sqlite::CreateTableSql;
#[derive(CreateTableSql)]
struct Dog {
	name: String,
}
assert_eq!(
	Dog::CREATE_TABLE_SQL,
	"CREATE TABLE IF NOT EXISTS dog (name TEXT NOT NULL) STRICT;"
);
# assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(Dog::CREATE_TABLE_SQL).is_ok())
```

column constraint
```rust
# use wb_sqlite::CreateTableSql;
#[derive(CreateTableSql)]
struct Cat {
	#[sql(constraint = "UNIQUE")]
	name: String,
	weight: Option<f64>
}
assert_eq!(
	Cat::CREATE_TABLE_SQL,
	"CREATE TABLE IF NOT EXISTS cat (name TEXT NOT NULL UNIQUE, weight REAL) STRICT;"
);
# assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(Cat::CREATE_TABLE_SQL).is_ok())
```

column datatype override
```rust
# use wb_sqlite::CreateTableSql;
#[derive(CreateTableSql)]
struct Human {
	#[sql(constraint = "PRIMARY KEY")]
	id: i64,
	#[sql(constraint = "CHECK(name != '')")]
	name: String,
	image: Option<Vec<u8>>,
	#[sql(typ = "ANY")]
	data: Option<Vec<u8>>,
}
assert_eq!(
	Human::CREATE_TABLE_SQL,
	concat!(
	"CREATE TABLE IF NOT EXISTS human (id INTEGER NOT NULL PRIMARY KEY, ",
	"name TEXT NOT NULL CHECK(name != ''), image BLOB, data ANY) STRICT;"
	)
);
# assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(Human::CREATE_TABLE_SQL).is_ok())
```

table constraint
```rust
# use wb_sqlite::CreateTableSql;
#[derive(CreateTableSql)]
#[sql(constraint = "UNIQUE(owner,name)")]
struct Pet {
	#[sql(constraint = "PRIMARY KEY")]
	id: i64,
	#[sql(constraint = "REFERENCES human(id) ON UPDATE RESTRICT ON DELETE RESTRICT")]
	owner: i64,
	name: String,
}
assert_eq!(
	Pet::CREATE_TABLE_SQL,
	concat!(
	"CREATE TABLE IF NOT EXISTS pet (id INTEGER NOT NULL PRIMARY KEY, ",
	"owner INTEGER NOT NULL REFERENCES human(id) ON UPDATE RESTRICT ON DELETE RESTRICT, ",
	"name TEXT NOT NULL, UNIQUE(owner,name)) STRICT;"
	)
);
# assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(Pet::CREATE_TABLE_SQL).is_ok())
```


### Insert + Update

sync rusqlite
```rust
# use wb_sqlite::{CreateTableSql,InsertSync,UpdateSync};
#[derive(CreateTableSql,InsertSync,UpdateSync)]
struct Person {
	#[sql(constraint = "PRIMARY KEY")]
	id: i64,
	#[sql(constraint = "UNIQUE")]
	name: String,
}

fn main() -> Result<(), rusqlite::Error> {
	let conn = rusqlite::Connection::open_in_memory()?;
	conn.execute_batch(Person::CREATE_TABLE_SQL)?;

	let p = Person {
		id: 0,
		name: "me".to_owned(),
	};
	let id = p.insert_sync(&conn)?;
	assert!(id > 0);

	let p2 = Person {
		id: id,
		name: "you".to_owned()
	};
	let ok = p2.update_sync(&conn)?;
	assert!(ok);

	Ok(())
}
```

async sqlx
```rust
# use wb_sqlite::{CreateTableSql,Get,Insert,Update};
#[derive(CreateTableSql,Get,Insert,Update,sqlx::FromRow)]
struct Person {
	#[sql(constraint = "PRIMARY KEY")]
	id: i64,
	#[sql(constraint = "UNIQUE")]
	name: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), sqlx::Error> {
	use sqlx::{Connection, Executor};
	let mut conn = sqlx::SqliteConnection::connect(":memory:").await?;
	conn.execute(Person::CREATE_TABLE_SQL).await?;

	let p = Person {
		id: 0,
		name: "me".to_owned(),
	};
	let id = p.insert(&mut conn).await?;
	assert!(id > 0);

	let p2 = Person {
		id: id,
		name: "you".to_owned()
	};
	let ok = p2.update(&mut conn).await?;
	assert!(ok);

	let p3 = Person::get_by_id(1,&mut conn).await?;
	assert_eq!(p3.name,"you");

	let p4 = Person::get_by_name("you",&mut conn).await?;
	assert_eq!(p4.id,1);

	Ok(())
}
```
