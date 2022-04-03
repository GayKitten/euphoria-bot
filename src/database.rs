use rusqlite::Connection;

struct UserIp {
	user: UserId
}

fn assure_tables(conn: &Connection) {
	conn.execute(include_str!("tables.sql"))
}
