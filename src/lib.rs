#![cfg_attr(doc, doc = include_str!("../README.md"))]

mod create_index;
mod create_table;
mod create_table_log;
mod get;
mod insert;
mod insert_sync;
mod select;
mod select_as;
mod update;
mod update_sync;
mod util;

use virtue::prelude::TokenStream;

/// const CREATE_INDEX_SQL: &'static str = "CREATE INDEX ..."
///
/// Create index for every column with a [foreign-key-clause](https://www.sqlite.org/syntax/foreign-key-clause.html).\
/// Works only if constraint is in all caps, lowercase serves as escape hatch.
///
/// ```rust
/// # use wb_sqlite::CreateIndexSql;
/// #[derive(CreateIndexSql)]
/// struct Cat {
///    #[sql(constraint = "PRIMARY KEY")]
///    name: String,
///    #[sql(constraint = "REFERENCES parent(id) ON UPDATE RESTRICT ON DELETE RESTRICT")]
///    mother: i64,
///    // lowercase first char prevents generation
///    #[sql(constraint = "rEFERENCES parent(id) ON UPDATE RESTRICT ON DELETE RESTRICT")]
///    father: i64,
///    #[sql(constraint = "REFERENCES human(id) ON UPDATE RESTRICT ON DELETE RESTRICT")]
///    owner: i64,
/// }
/// assert_eq!(
///    Cat::CREATE_INDEX_SQL,
///    concat!(
///    "CREATE INDEX IF NOT EXISTS cat_mother_idx ON cat(mother); ",
///    "CREATE INDEX IF NOT EXISTS cat_owner_idx ON cat(owner); "
///    )
/// );
/// ```
#[proc_macro_derive(CreateIndexSql, attributes(sql))]
pub fn create_index(input: TokenStream) -> TokenStream {
	create_index::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}

/// const CREATE_TABLE_SQL: &'static str = "CREATE TABLE ..."
///
/// `"CREATE TABLE IF NOT EXISTS {tab_name} ({col_defs}{tab_constraint}) STRICT{tab_option};"`\
/// `col_defs = {field_name} {col_typ} {col_constraint},`
///
/// ## Struct attributes
///
/// #[sql(
/// constraint = "[table constraint](https://www.sqlite.org/syntax/table-constraint.html)",
/// option = "[table option](https://www.sqlite.org/syntax/table-options.html)"
/// )]
///
/// ## Field attributes
///
/// #[sql(
/// typ = "[datatype](https://www.sqlite.org/stricttables.html)",
/// constraint = "[column constraint](https://www.sqlite.org/syntax/column-constraint.html)"
/// )]
///
/// ## Table Name creation: PascalCase with digits as lowercase to snake_case
///
/// ```rust
/// # use wb_sqlite::CreateTableSql;
/// # #[derive(CreateTableSql)]
/// struct MyDog {
/// #    name: String,
/// # }
/// # assert_eq!(
/// # MyDog::CREATE_TABLE_SQL,
/// "CREATE TABLE IF NOT EXISTS my_dog (name TEXT NOT NULL) STRICT;"
/// # );
/// # assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(MyDog::CREATE_TABLE_SQL).is_ok());
///
/// # #[derive(CreateTableSql)]
/// struct M2yDog {
/// #    name: String,
/// # }
/// # assert_eq!(
/// # M2yDog::CREATE_TABLE_SQL,
/// "CREATE TABLE IF NOT EXISTS m2y_dog (name TEXT NOT NULL) STRICT;"
/// # );
/// # assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(M2yDog::CREATE_TABLE_SQL).is_ok());
///
/// # #[derive(CreateTableSql)]
/// struct My2Dog {
/// #    name: String,
/// # }
/// # assert_eq!(
/// # My2Dog::CREATE_TABLE_SQL,
/// "CREATE TABLE IF NOT EXISTS my2_dog (name TEXT NOT NULL) STRICT;"
/// # );
/// # assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(My2Dog::CREATE_TABLE_SQL).is_ok());
///
/// # #[derive(CreateTableSql)]
/// struct MyD2og {
/// #    name: String,
/// # }
/// # assert_eq!(
/// # MyD2og::CREATE_TABLE_SQL,
/// "CREATE TABLE IF NOT EXISTS my_d2og (name TEXT NOT NULL) STRICT;"
/// # );
/// # assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(MyD2og::CREATE_TABLE_SQL).is_ok());
/// # #[derive(CreateTableSql)]
///
/// struct MyDo2g {
/// #    name: String,
/// # }
/// # assert_eq!(
/// # MyDo2g::CREATE_TABLE_SQL,
/// "CREATE TABLE IF NOT EXISTS my_do2g (name TEXT NOT NULL) STRICT;"
/// # );
/// # assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(MyDo2g::CREATE_TABLE_SQL).is_ok());
/// ```
///
/// ## Example
///
/// ```rust
/// # use wb_sqlite::CreateTableSql;
/// #[derive(CreateTableSql)]
/// #[sql(constraint = "UNIQUE(vendor,brand)")]
/// struct WineBottle {
///    #[sql(constraint = "PRIMARY KEY")]
///    id: i64,
///    #[sql(constraint = "UNIQUE")]
///    serial_no: Option<String>,
///    #[sql(constraint = "REFERENCES vendor(id) ON UPDATE RESTRICT ON DELETE RESTRICT")]
///    vendor: i64,
///    #[sql(constraint = "CHECK(volume > 0)")]
///    volume: f64,
///    #[sql(constraint = "DEFAULT 'red'")]
///    color: String,
///    brand: Option<String>,
///    #[sql(typ = "ANY")]
///    data: Option<Vec<u8>>
/// }
/// assert_eq!(
/// WineBottle::CREATE_TABLE_SQL,
/// concat!(
/// "CREATE TABLE IF NOT EXISTS wine_bottle (id INTEGER NOT NULL PRIMARY KEY, ",
/// "serial_no TEXT UNIQUE, ",
/// "vendor INTEGER NOT NULL REFERENCES vendor(id) ON UPDATE RESTRICT ON DELETE RESTRICT, ",
/// "volume REAL NOT NULL CHECK(volume > 0), ",
/// "color TEXT NOT NULL DEFAULT 'red', ",
/// "brand TEXT, data ANY, UNIQUE(vendor,brand)) STRICT;",
/// ));
/// # assert!(rusqlite::Connection::open_in_memory().unwrap().execute_batch(WineBottle::CREATE_TABLE_SQL).is_ok());
/// ```
///
/// ## SQLite / Rust type mapping
///
/// `INTEGER NOT NULL = bool, u8, u16, u32, i8, i16, i32, i64`\
/// `REAL NOT NULL    = f32, f64`\
/// `TEXT NOT NULL    = &str, String`\
/// `BLOB NOT NULL    = &[u8], Vec<u8>`\
/// `INTEGER          = Option<bool>, Option<u8>, Option<u16> ... Option<i64>`\
/// `REAL             = Option<f32>, Option<f64>`\
/// `TEXT             = Option<String>`\
/// `BLOB             = Option<Vec<u8>>`\
/// `ANY              = all other`
#[proc_macro_derive(CreateTableSql, attributes(sql))]
pub fn create_table(input: TokenStream) -> TokenStream {
	create_table::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}

/// const CREATE_TABLE_LOG_SQL: &'static str = "CREATE ..."
///
/// Create logging-table + trigger to log all table row modifications to the logging-table.
///
/// ```rust
/// # use wb_sqlite::CreateTableLogSql;
/// #[derive(CreateTableLogSql)]
/// struct FavoritePet {
///    #[sql(constraint = "PRIMARY KEY")]
///    id: i64,
///    name: String,
/// }
/// assert_eq!(
///    FavoritePet::CREATE_TABLE_LOG_SQL,
///    concat!(
///    "CREATE TABLE IF NOT EXISTS favorite_pet_log (id INTEGER NOT NULL, name TEXT NOT NULL) STRICT; ",
///    "CREATE INDEX IF NOT EXISTS favorite_pet_log_id_idx ON favorite_pet_log(id); ",
///    "CREATE TRIGGER IF NOT EXISTS favorite_pet_update UPDATE ON favorite_pet ",
///    "BEGIN INSERT INTO favorite_pet_log (id,name) VALUES (OLD.id,OLD.name); END; ",
///    "CREATE TRIGGER IF NOT EXISTS favorite_pet_delete DELETE ON favorite_pet ",
///    "BEGIN INSERT INTO favorite_pet_log (id,name) VALUES (OLD.id,OLD.name); END;"
///    )
/// );
/// ```
#[proc_macro_derive(CreateTableLogSql, attributes(sql))]
pub fn create_table_log(input: TokenStream) -> TokenStream {
	create_table_log::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}

/// const SELECT_SQL : &'static str = "SELECT ..."
///
/// "SELECT {columns} FROM {tab_name}"
///
/// ```rust
/// # use wb_sqlite::SelectSql;
/// #[derive(SelectSql)]
/// struct Car {
///    #[sql(constraint = "PRIMARY KEY")]
///    id: i64,
///    model: String,
///    length: f64,
///    height: f64,
///    width: f64,
/// }
/// assert_eq!(
///    Car::SELECT_SQL,
///    "SELECT id,model,length,height,width FROM car"
/// );
/// ```
#[proc_macro_derive(SelectSql, attributes(sql))]
pub fn select(input: TokenStream) -> TokenStream {
	select::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}

/// const SELECT_AS_SQL : &'static str = "SELECT ..."
///
/// Create a SELECT Statment with aliasing via AS.
///
/// ## Struct attributes
///
/// #[sqlas(from = "[select stmt after from](https://www.sqlite.org/lang_select.html)")]
///
/// if omitted defaults to {tab_name}
///
/// ## Field attributes
///
/// #[sqlas(col = "result column")]
///
/// if ommited defaults to {col_name}
///
/// ```rust
/// # use wb_sqlite::SelectAsSql;
/// #[derive(SelectAsSql)]
/// #[sqlas(from = "old_table1 AS t1 INNER JOIN old_table2 AS t2 on t1.pk = t2.fk")]
/// struct NewTable {
///    id: i64,
///    #[sqlas(col = "t1.moniker")]
///    name: String,
///    #[sqlas(col = "t2.description")]
///    info: String,
///    note: String,
///    #[sqlas(col = "t1.pressure_psi * 0.068947573")]
///    pressure_bar: f64,
/// }
/// assert_eq!(
///    NewTable::SELECT_AS_SQL,
///    concat!(
///    "SELECT id,t1.moniker AS name,t2.description AS info,note,",
///    "t1.pressure_psi * 0.068947573 AS pressure_bar FROM ",
///    "old_table1 AS t1 INNER JOIN old_table2 AS t2 on t1.pk = t2.fk"
///    )
/// );
/// ```
#[proc_macro_derive(SelectAsSql, attributes(sqlas))]
pub fn select_as(input: TokenStream) -> TokenStream {
	select_as::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}

/// fn get_by_{field-name}({field-name}: {field-type}, exec: impl sqlx::SqliteExecutor<'_ >) -> Result<Self, sqlx::Error>
///
/// Generate fn for PRIMARY KEY and every UNIQUE constraint.\
/// Works only if constraint is in all caps, lowercase serves as escape hatch.
///
/// ```rust
/// # use wb_sqlite::{Get,CreateTableSql,Insert};
/// #[derive(CreateTableSql,Get,Insert,sqlx::FromRow)]
/// struct Cat {
///    #[sql(constraint = "PRIMARY KEY")]
///    id: i64,
///    #[sql(constraint = "UNIQUE")]
///    name: String,
///    #[sql(constraint = "uNIQUE")]
///    owner: String,
/// }
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() -> Result<(), sqlx::Error> {
///    use sqlx::{Connection, Executor};
///    let mut conn = sqlx::SqliteConnection::connect(":memory:").await?;
///    conn.execute(Cat::CREATE_TABLE_SQL).await?;
///
///    let c = Cat {
///       id: 0,
///       name: "meouw".to_owned(),
///       owner: "nobody".to_owned()
///    };
///    let id = c.insert(&mut conn).await?;
///    assert!(id > 0);
///
///    let c2 = Cat::get_by_id(1,&mut conn).await?;
///    assert_eq!(c2.name,"meouw");
///    let c3 = Cat::get_by_name("meouw",&mut conn).await?;
///    assert_eq!(c3.owner,"nobody");
///
///    // there is no fn get_by_owner
///    // because the constraint contains lowercase-chars
///
///    Ok(())
/// }
/// ```
#[proc_macro_derive(Get, attributes(sql))]
pub fn get(input: TokenStream) -> TokenStream {
	get::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}

/// fn insert(&self, exec: impl sqlx::SqliteExecutor<'_ >) -> Result<i64, sqlx::Error>
///
/// Generate fn for INSERT with sqlx.
///
/// If there is a PRIMARY KEY and the rust-type is i64 then the insert depends on the value of the pk.\
/// If pk > 0 then do a full insert including the pk, else do a insert without the pk so sqlite assigns a new one,
/// which is returned as result.
///
/// Without PRIMARY KEY or rust-type != i64 do a full insert and return the last_insert_rowid.
///
/// PRIMARY KEY detection works only if constraint is in all caps, lowercase serves as escape hatch.
///
/// ```rust
/// # use wb_sqlite::{CreateTableSql,Insert};
/// #[derive(CreateTableSql,Insert)]
/// struct Cat {
///    #[sql(constraint = "PRIMARY KEY")]
///    id: i64,
///    name: String,
/// }
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() -> Result<(), sqlx::Error> {
///    use sqlx::{Connection, Executor};
///    let mut conn = sqlx::SqliteConnection::connect(":memory:").await?;
///    conn.execute(Cat::CREATE_TABLE_SQL).await?;
///
///    let c = Cat {
///       id: 0,
///       name: "miau".to_owned(),
///    };
///    let id = c.insert(&mut conn).await?;
///    assert!(id == 1);
///
///    Ok(())
/// }
/// ```
#[proc_macro_derive(Insert, attributes(sql))]
pub fn insert(input: TokenStream) -> TokenStream {
	insert::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}

/// fn insert_sync(&self, conn: &rusqlite::Connection) -> Result<i64, rusqlite::Error>
///
/// Generate fn for INSERT with rusqlite.
///
/// If there is a PRIMARY KEY and the rust-type is i64 then the insert depends on the value of the pk.\
/// If pk > 0 then do a full insert including the pk, else do a insert without the pk so sqlite assigns a new one,
/// which is returned as result.
///
/// Without PRIMARY KEY or rust-type != i64 do a full insert and return the last_insert_rowid.
///
/// PRIMARY KEY detection works only if constraint is in all caps, lowercase serves as escape hatch.
///
/// ```rust
/// # use wb_sqlite::{CreateTableSql,InsertSync};
/// #[derive(CreateTableSql,InsertSync)]
/// struct Cat {
///    #[sql(constraint = "PRIMARY KEY")]
///    id: i64,
///    name: String,
/// }
/// fn main() -> Result<(), rusqlite::Error> {
///    let conn = rusqlite::Connection::open_in_memory()?;
///    conn.execute_batch(Cat::CREATE_TABLE_SQL)?;
///
///    let c = Cat {
///       id: 0,
///       name: "miau".to_owned(),
///    };
///    let id = c.insert_sync(&conn)?;
///    assert!(id == 1);
///
///    Ok(())
/// }
/// ```
#[proc_macro_derive(InsertSync, attributes(sql))]
pub fn insert_sync(input: TokenStream) -> TokenStream {
	insert_sync::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}

/// fn update(&self, exec: impl sqlx::SqliteExecutor<'_ >) -> Result<bool, sqlx::Error>
///
/// Generate fn for UPDATE with sqlx, if there is a PRIMARY KEY.\
/// PRIMARY KEY detection works only if constraint is in all caps, lowercase serves as escape hatch.
///
/// `UPDATE {tab_name} SET {cols} WHERE {pk}=`
///
/// ```rust
/// # use wb_sqlite::{CreateTableSql,Insert,Update};
/// #[derive(CreateTableSql,Insert,Update)]
/// struct Cat {
///    #[sql(constraint = "PRIMARY KEY")]
///    id: i64,
///    name: String,
/// }
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() -> Result<(), sqlx::Error> {
///    use sqlx::{Connection, Executor};
///    let mut conn = sqlx::SqliteConnection::connect(":memory:").await?;
///    conn.execute(Cat::CREATE_TABLE_SQL).await?;
///
///    let c = Cat {
///       id: 0,
///       name: "miau".to_owned(),
///    };
///    let id = c.insert(&mut conn).await?;
///    assert!(id == 1);
///
///    let c2 = Cat {
///       id: 1,
///       name: "meouw".to_owned(),
///    };
///    let ok = c2.update(&mut conn).await?;
///    assert!(ok);
///
///    Ok(())
/// }
/// ```
#[proc_macro_derive(Update, attributes(sql))]
pub fn update(input: TokenStream) -> TokenStream {
	update::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}

/// fn update_sync(&self, conn: &rusqlite::Connection) -> Result<bool, rusqlite::Error>
///
/// Generate fn for UPDATE with rusqlite, if there is a PRIMARY KEY.\
/// PRIMARY KEY detection works only if constraint is in all caps, lowercase serves as escape hatch.
///
/// `UPDATE {tab_name} SET {cols} WHERE {pk}=`
///
/// ```rust
/// # use wb_sqlite::{CreateTableSql,InsertSync,UpdateSync};
/// #[derive(CreateTableSql,InsertSync,UpdateSync)]
/// struct Cat {
///    #[sql(constraint = "PRIMARY KEY")]
///    id: i64,
///    name: String,
/// }
/// fn main() -> Result<(), rusqlite::Error> {
///    let conn = rusqlite::Connection::open_in_memory()?;
///    conn.execute_batch(Cat::CREATE_TABLE_SQL)?;
///
///    let c = Cat {
///       id: 0,
///       name: "miau".to_owned(),
///    };
///    let id = c.insert_sync(&conn)?;
///    assert!(id == 1);
///
///    let c2 = Cat {
///       id: 1,
///       name: "meouw".to_owned(),
///    };
///    let ok = c2.update_sync(&conn)?;
///    assert!(ok);
///
///    Ok(())
/// }
/// ```
#[proc_macro_derive(UpdateSync, attributes(sql))]
pub fn update_sync(input: TokenStream) -> TokenStream {
	update_sync::inner(input).unwrap_or_else(virtue::Error::into_token_stream)
}
