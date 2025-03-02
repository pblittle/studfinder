use crate::Piece;
use crate::error::{Result, StudFinderError};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;
use tracing::{debug, info};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Creates a new Database instance with the specified path
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to open the database connection
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        debug!("Opening database at: {:?}", path.as_ref());
        let conn = Connection::open(path)
            .map_err(|e| StudFinderError::Database(e))?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        Ok(db)
    }

    /// Initializes the database schema, creating tables and applying migrations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to start a transaction
    /// - Failed to create or modify database tables
    /// - Failed to commit the transaction
    pub fn init(&self) -> Result<()> {
        debug!("Initializing database schema");

        // Acquire lock and start transaction in two steps
        let mut conn = self.conn.lock()?;
        
        let tx = conn.transaction()
            .map_err(|e| StudFinderError::Database(e))?;

        // Create schema version table
        tx.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| StudFinderError::Database(e))?;

        // Get current schema version
        let version: i32 = tx
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| StudFinderError::Database(e))?;

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
            )
            .map_err(|e| StudFinderError::DatabaseInitFailed(e.to_string()))?;

            tx.execute("INSERT INTO schema_version (version) VALUES (1)", [])
                .map_err(|e| StudFinderError::DatabaseInitFailed(e.to_string()))?;
        }

        if version < 2 {
            debug!("Applying migration to version 2: Adding confidence column");
            tx.execute(
                "ALTER TABLE pieces ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0",
                [],
            )
            .map_err(|e| StudFinderError::DatabaseInitFailed(e.to_string()))?;

            tx.execute(
                "CREATE INDEX IF NOT EXISTS idx_part_number ON pieces(part_number)",
                [],
            )
            .map_err(|e| StudFinderError::DatabaseInitFailed(e.to_string()))?;

            tx.execute("CREATE INDEX IF NOT EXISTS idx_color ON pieces(color)", [])
                .map_err(|e| StudFinderError::DatabaseInitFailed(e.to_string()))?;

            tx.execute("INSERT INTO schema_version (version) VALUES (2)", [])
                .map_err(|e| StudFinderError::DatabaseInitFailed(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| StudFinderError::DatabaseInitFailed(format!("Failed to commit schema changes: {}", e)))?;
        debug!(
            "Database schema initialized successfully to version {}",
            self.get_schema_version()?
        );
        Ok(())
    }

    /// Resets the database schema, dropping all tables and reinitializing
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to start a transaction
    /// - Failed to drop tables
    /// - Failed to commit the transaction
    /// - Failed to reinitialize the schema
    pub fn reset(&self) -> Result<()> {
        info!("Resetting database schema");

        {
            // Acquire lock and start transaction in two steps
            let mut conn = self.conn.lock()?;
            
            let tx = conn.transaction()
                .map_err(|e| StudFinderError::Database(e))?;

            tx.execute("DROP TABLE IF EXISTS pieces", [])
                .map_err(|e| StudFinderError::Database(e))?;
            tx.execute("DROP TABLE IF EXISTS schema_version", [])
                .map_err(|e| StudFinderError::Database(e))?;

            tx.commit()
                .map_err(|e| StudFinderError::Database(e))?;
        } // Release the lock before calling init

        self.init()?;

        info!("Database schema reset complete");
        Ok(())
    }

    /// Adds a piece to the database or updates its quantity if it already exists
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to start a transaction
    /// - Failed to query, insert, or update the piece
    /// - Failed to commit the transaction
    pub fn add_piece(&self, piece: &Piece) -> Result<()> {
        debug!("Adding piece to database: {}", piece);

        // Acquire lock and start transaction in two steps
        let mut conn = self.conn.lock()?;
        
        let tx = conn.transaction()
            .map_err(|e| StudFinderError::Database(e))?;

        let existing = {
            let mut stmt = tx
                .prepare(
                    "SELECT id, part_number, color, category, quantity, confidence
                 FROM pieces WHERE id = ?",
                )
                .map_err(|e| StudFinderError::Database(e))?;

            stmt.query_row([&piece.id], |row| {
                Ok(Piece {
                    id: row.get(0)?,
                    part_number: row.get(1)?,
                    color: row.get(2)?,
                    category: row.get(3)?,
                    quantity: row.get(4)?,
                    confidence: row.get(5)?,
                })
            })
            .optional()
            .map_err(|e| StudFinderError::Database(e))?
        };

        if let Some(existing_piece) = existing {
            debug!("Found existing piece, updating quantity");
            tx.execute(
                "UPDATE pieces SET quantity = ?1 WHERE id = ?2",
                params![piece.quantity + existing_piece.quantity, piece.id],
            )
            .map_err(|e| StudFinderError::Database(e))?;
        } else {
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
            )
            .map_err(|e| StudFinderError::Database(e))?;
        }

        tx.commit()
            .map_err(|e| StudFinderError::Database(e))?;
        debug!("Successfully added/updated piece in database");

        Ok(())
    }

    /// Retrieves a piece from the database by its ID
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to prepare or execute the query
    pub fn get_piece(&self, id: &str) -> Result<Option<Piece>> {
        debug!("Fetching piece with id: {}", id);

        let conn = self.conn.lock()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, part_number, color, category, quantity, confidence
             FROM pieces WHERE id = ?",
            )
            .map_err(|e| StudFinderError::Database(e))?;

        let piece = stmt
            .query_row([id], |row| {
                Ok(Piece {
                    id: row.get(0)?,
                    part_number: row.get(1)?,
                    color: row.get(2)?,
                    category: row.get(3)?,
                    quantity: row.get(4)?,
                    confidence: row.get(5)?,
                })
            })
            .optional()
            .map_err(|e| StudFinderError::Database(e))?;

        debug!("Piece lookup result: {:?}", piece);
        Ok(piece)
    }

    /// Lists all pieces in the inventory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to prepare or execute the query
    /// - Failed to collect the results
    pub fn list_pieces(&self) -> Result<Vec<Piece>> {
        debug!("Listing all pieces in inventory");

        let conn = self.conn.lock()?;

        let mut stmt = conn
            .prepare("SELECT id, part_number, color, category, quantity, confidence FROM pieces")
            .map_err(|e| StudFinderError::Database(e))?;

        let pieces = stmt
            .query_map([], |row| {
                Ok(Piece {
                    id: row.get(0)?,
                    part_number: row.get(1)?,
                    color: row.get(2)?,
                    category: row.get(3)?,
                    quantity: row.get(4)?,
                    confidence: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| StudFinderError::Database(e))?;

        debug!("Found {} pieces in inventory", pieces.len());
        Ok(pieces)
    }

    /// Updates the quantity of a piece in the database
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to execute the update
    pub fn update_quantity(&self, id: &str, quantity: i32) -> Result<()> {
        debug!("Updating quantity for piece {}: {}", id, quantity);

        let conn = self.conn.lock()?;

        conn.execute(
            "UPDATE pieces SET quantity = ?1 WHERE id = ?2",
            params![quantity, id],
        )
        .map_err(|e| StudFinderError::Database(e))?;

        Ok(())
    }

    /// Deletes a piece from the database
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to execute the delete
    pub fn delete_piece(&self, id: &str) -> Result<()> {
        debug!("Deleting piece with id: {}", id);

        let conn = self.conn.lock()?;

        conn.execute("DELETE FROM pieces WHERE id = ?", [id])
            .map_err(|e| StudFinderError::Database(e))?;

        Ok(())
    }

    /// Gets the current schema version
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to query the schema version
    fn get_schema_version(&self) -> Result<i32> {
        let conn = self.conn.lock()?;

        let version: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| StudFinderError::Database(e))?;

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
