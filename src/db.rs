use crate::Piece;
use anyhow::{Result, Context};
use rusqlite::{Connection, params, OptionalExtension};
use std::path::Path;
use tracing::{debug, info};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        debug!("Opening database at: {:?}", path.as_ref());
        let conn = Connection::open(path)
            .context("Failed to open database connection")?;
        let db = Self {
            conn: Mutex::new(conn)
        };
        Ok(db)
    }

    pub fn init(&self) -> Result<()> {
        debug!("Initializing database schema");

        let mut conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;

        // Start a transaction for schema initialization
        let tx = conn.transaction()
            .context("Failed to start transaction")?;

        // Create schema version table
        tx.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).context("Failed to create schema version table")?;

        // Get current schema version
        let version: i32 = tx.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0)
        ).context("Failed to get schema version")?;

        debug!("Current schema version: {}", version);

        // Apply migrations based on version
        if version < 1 {
            debug!("Applying migration to version 1");
            tx.execute(
                "CREATE TABLE IF NOT EXISTS pieces (
                    id TEXT PRIMARY KEY,
                    part_number TEXT NOT NULL,
                    color TEXT NOT NULL,
                    category TEXT NOT NULL,
                    quantity INTEGER NOT NULL DEFAULT 1
                )",
                [],
            ).context("Failed to create pieces table")?;

            tx.execute(
                "INSERT INTO schema_version (version) VALUES (1)",
                [],
            ).context("Failed to update schema version to 1")?;
        }

        if version < 2 {
            debug!("Applying migration to version 2: Adding confidence column");
            tx.execute(
                "ALTER TABLE pieces ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0",
                [],
            ).context("Failed to add confidence column")?;

            tx.execute(
                "CREATE INDEX IF NOT EXISTS idx_part_number ON pieces(part_number)",
                [],
            ).context("Failed to create part_number index")?;

            tx.execute(
                "CREATE INDEX IF NOT EXISTS idx_color ON pieces(color)",
                [],
            ).context("Failed to create color index")?;

            tx.execute(
                "INSERT INTO schema_version (version) VALUES (2)",
                [],
            ).context("Failed to update schema version to 2")?;
        }

        tx.commit().context("Failed to commit schema changes")?;
        debug!("Database schema initialized successfully to version {}", self.get_schema_version()?);
        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        info!("Resetting database schema");

        let mut conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;

        let tx = conn.transaction()
            .context("Failed to start transaction for reset")?;

        tx.execute("DROP TABLE IF EXISTS pieces", [])
            .context("Failed to drop pieces table")?;
        tx.execute("DROP TABLE IF EXISTS schema_version", [])
            .context("Failed to drop schema_version table")?;

        tx.commit().context("Failed to commit schema reset")?;

        self.init().context("Failed to reinitialize schema")?;

        info!("Database schema reset complete");
        Ok(())
    }

    pub fn add_piece(&self, piece: &Piece) -> Result<()> {
        debug!("Adding piece to database: {}", piece);

        let mut conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;

        // Start a transaction for the entire operation
        let tx = conn.transaction()
            .context("Failed to start transaction")?;

        let existing = {
            let mut stmt = tx.prepare(
                "SELECT id, part_number, color, category, quantity, confidence
                 FROM pieces WHERE id = ?"
            ).context("Failed to prepare get_piece statement")?;

            stmt.query_row([&piece.id], |row| {
                Ok(Piece {
                    id: row.get(0)?,
                    part_number: row.get(1)?,
                    color: row.get(2)?,
                    category: row.get(3)?,
                    quantity: row.get(4)?,
                    confidence: row.get(5)?,
                })
            }).optional().context("Failed to query piece")?
        };

        match existing {
            Some(existing_piece) => {
                debug!("Found existing piece, updating quantity");
                tx.execute(
                    "UPDATE pieces SET quantity = ?1 WHERE id = ?2",
                    params![
                        piece.quantity + existing_piece.quantity,
                        piece.id
                    ],
                ).context("Failed to update quantity")?;
            },
            None => {
                debug!("Inserting new piece");
                tx.execute(
                    "INSERT INTO pieces (id, part_number, color, category, quantity, confidence)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        piece.id,
                        piece.part_number,
                        piece.color,
                        piece.category,
                        piece.quantity,
                        piece.confidence
                    ],
                ).context("Failed to insert piece")?;
            }
        }

        tx.commit().context("Failed to commit transaction")?;
        debug!("Successfully added/updated piece in database");

        Ok(())
    }

    pub fn get_piece(&self, id: &str) -> Result<Option<Piece>> {
        debug!("Fetching piece with id: {}", id);

        let conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;

        let mut stmt = conn.prepare(
            "SELECT id, part_number, color, category, quantity, confidence
             FROM pieces WHERE id = ?"
        ).context("Failed to prepare get_piece statement")?;

        let piece = stmt.query_row([id], |row| {
            Ok(Piece {
                id: row.get(0)?,
                part_number: row.get(1)?,
                color: row.get(2)?,
                category: row.get(3)?,
                quantity: row.get(4)?,
                confidence: row.get(5)?,
            })
        }).optional().context("Failed to query piece")?;

        debug!("Piece lookup result: {:?}", piece);
        Ok(piece)
    }

    pub fn list_pieces(&self) -> Result<Vec<Piece>> {
        debug!("Listing all pieces in inventory");

        let conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;

        let mut stmt = conn.prepare(
            "SELECT id, part_number, color, category, quantity, confidence FROM pieces"
        ).context("Failed to prepare list_pieces statement")?;

        let pieces = stmt.query_map([], |row| {
            Ok(Piece {
                id: row.get(0)?,
                part_number: row.get(1)?,
                color: row.get(2)?,
                category: row.get(3)?,
                quantity: row.get(4)?,
                confidence: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()
        .context("Failed to collect pieces")?;

        debug!("Found {} pieces in inventory", pieces.len());
        Ok(pieces)
    }

    pub fn update_quantity(&self, id: &str, quantity: i32) -> Result<()> {
        debug!("Updating quantity for piece {}: {}", id, quantity);

        let conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;

        conn.execute(
            "UPDATE pieces SET quantity = ?1 WHERE id = ?2",
            params![quantity, id],
        ).context("Failed to update quantity")?;

        Ok(())
    }

    pub fn delete_piece(&self, id: &str) -> Result<()> {
        debug!("Deleting piece with id: {}", id);

        let conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;

        conn.execute(
            "DELETE FROM pieces WHERE id = ?",
            [id],
        ).context("Failed to delete piece")?;

        Ok(())
    }

    fn get_schema_version(&self) -> Result<i32> {
        let conn = self.conn.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire database lock"))?;

        let version: i32 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0)
        ).context("Failed to get schema version")?;

        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_piece() -> Piece {
        Piece {
            id: String::from("test-piece"),
            part_number: "3001".to_string(),
            color: "Red".to_string(),
            category: "Brick".to_string(),
            quantity: 1,
            confidence: 0.95,
        }
    }

    #[test]
    fn test_database_operations() {
        let db = Database::new(":memory:").unwrap();
        db.init().unwrap();

        // Test schema version
        assert_eq!(db.get_schema_version().unwrap(), 2);

        // Test insert
        let piece = create_test_piece();
        db.add_piece(&piece).unwrap();

        // Test get
        let fetched = db.get_piece(&piece.id).unwrap().unwrap();
        assert_eq!(fetched.part_number, piece.part_number);
        assert_eq!(fetched.confidence, piece.confidence);

        // Test list
        let pieces = db.list_pieces().unwrap();
        assert_eq!(pieces.len(), 1);

        // Test update
        db.update_quantity(&piece.id, 2).unwrap();
        let updated = db.get_piece(&piece.id).unwrap().unwrap();
        assert_eq!(updated.quantity, 2);

        // Test delete
        db.delete_piece(&piece.id).unwrap();
        let pieces = db.list_pieces().unwrap();
        assert_eq!(pieces.len(), 0);
    }

    #[test]
    fn test_schema_reset() {
        let db = Database::new(":memory:").unwrap();

        // Initial setup
        db.init().unwrap();
        let piece = create_test_piece();
        db.add_piece(&piece).unwrap();
        assert_eq!(db.list_pieces().unwrap().len(), 1);

        // Reset database
        db.reset().unwrap();
        assert_eq!(db.list_pieces().unwrap().len(), 0);
        assert_eq!(db.get_schema_version().unwrap(), 2);
    }
}