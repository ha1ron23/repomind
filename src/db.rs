use rusqlite::{Connection, Result};

pub struct IndexDb {
    pub conn: Connection,
}

impl IndexDb {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line INTEGER NOT NULL
            )",
            [],
        )?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_name ON symbols(name)", [])?;
        Ok(Self { conn })
    }

    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM symbols", [])?;
        Ok(())
    }

    pub fn insert_symbol(&self, name: &str, typ: &str, file_path: &str, line: u32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO symbols (name, type, file_path, line) VALUES (?1, ?2, ?3, ?4)",
            [name, typ, file_path, &line.to_string()],
        )?;
        Ok(())
    }

    pub fn find_symbol(&self, name: &str) -> Result<Vec<(String, String, u32)>> {
        let mut stmt = self.conn.prepare(
            "SELECT type, file_path, line FROM symbols WHERE name = ?1 ORDER BY file_path, line"
        )?;
        let rows = stmt.query_map([name], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}