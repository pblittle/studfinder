use crate::core::piece::Piece;
use crate::error::{Result, StudFinderError};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Database management for the StudFinder application
///
/// Handles all database operations, including schema management,
/// piece storage and retrieval, and inventory operations.
#[derive(Debug)]
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Creates a new Database instance with the specified path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file
    ///
    /// # Returns
    ///
    /// A new `Database` instance or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to open the database connection
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        debug!("Opening database at: {:?}", path.as_ref());
        let conn = Connection::open(path).map_err(|e| StudFinderError::Database {
            operation: "open connection".to_string(),
            source: Box::new(e),
        })?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        Ok(db)
    }

    /// Initializes the database schema, creating tables and applying migrations
    ///
    /// # Returns
    ///
    /// `Ok(())` if the schema was initialized successfully, or an error
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

        // Acquire lock and start transaction
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| StudFinderError::DatabaseLockFailed {
                operation: "initialize schema".to_string(),
            })?;

        let tx = conn.transaction().map_err(|e| StudFinderError::Database {
            operation: "begin transaction".to_string(),
            source: Box::new(e),
        })?;

        // Create schema version table
        tx.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| StudFinderError::Database {
            operation: "create schema_version table".to_string(),
            source: Box::new(e),
        })?;

        // Get current schema version
        let version: i32 = tx
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| StudFinderError::Database {
                operation: "query schema version".to_string(),
                source: Box::new(e),
            })?;

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
            .map_err(|e| StudFinderError::Migration {
                version: 1,
                operation: "create pieces table".to_string(),
                source: Box::new(e),
            })?;

            tx.execute("INSERT INTO schema_version (version) VALUES (1)", [])
                .map_err(|e| StudFinderError::Migration {
                    version: 1,
                    operation: "update schema version".to_string(),
                    source: Box::new(e),
                })?;
        }

        if version < 2 {
            debug!("Applying migration to version 2: Adding confidence column");
            tx.execute(
                "ALTER TABLE pieces ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0",
                [],
            )
            .map_err(|e| StudFinderError::Migration {
                version: 2,
                operation: "add confidence column".to_string(),
                source: Box::new(e),
            })?;

            tx.execute(
                "CREATE INDEX IF NOT EXISTS idx_part_number ON pieces(part_number)",
                [],
            )
            .map_err(|e| StudFinderError::Migration {
                version: 2,
                operation: "create part_number index".to_string(),
                source: Box::new(e),
            })?;

            tx.execute("CREATE INDEX IF NOT EXISTS idx_color ON pieces(color)", [])
                .map_err(|e| StudFinderError::Migration {
                    version: 2,
                    operation: "create color index".to_string(),
                    source: Box::new(e),
                })?;

            tx.execute("INSERT INTO schema_version (version) VALUES (2)", [])
                .map_err(|e| StudFinderError::Migration {
                    version: 2,
                    operation: "update schema version".to_string(),
                    source: Box::new(e),
                })?;
        }

        tx.commit().map_err(|e| StudFinderError::Database {
            operation: "commit transaction".to_string(),
            source: Box::new(e),
        })?;

        debug!(
            "Database schema initialized successfully to version {}",
            self.get_schema_version()?
        );
        Ok(())
    }

    /// Resets the database schema, dropping all tables and reinitializing
    ///
    /// # Returns
    ///
    /// `Ok(())` if the schema was reset successfully, or an error
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
            // Acquire lock and start transaction
            let mut conn = self
                .conn
                .lock()
                .map_err(|_| StudFinderError::DatabaseLockFailed {
                    operation: "reset schema".to_string(),
                })?;

            let tx = conn.transaction().map_err(|e| StudFinderError::Database {
                operation: "begin transaction".to_string(),
                source: Box::new(e),
            })?;

            tx.execute("DROP TABLE IF EXISTS pieces", []).map_err(|e| {
                StudFinderError::Database {
                    operation: "drop pieces table".to_string(),
                    source: Box::new(e),
                }
            })?;

            tx.execute("DROP TABLE IF EXISTS schema_version", [])
                .map_err(|e| StudFinderError::Database {
                    operation: "drop schema_version table".to_string(),
                    source: Box::new(e),
                })?;

            tx.commit().map_err(|e| StudFinderError::Database {
                operation: "commit transaction".to_string(),
                source: Box::new(e),
            })?;
        } // Release the lock before calling init

        match self.init() {
            Ok(_) => {
                info!("Database schema reset complete");
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize database after reset: {}", e);
                Err(StudFinderError::DatabaseResetFailed {
                    source: Box::new(e),
                })
            }
        }
    }

    /// Adds a piece to the database or updates its quantity if it already exists
    ///
    /// # Arguments
    ///
    /// * `piece` - The piece to add to the database
    ///
    /// # Returns
    ///
    /// `Ok(())` if the piece was added or updated successfully, or an error
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

        // Acquire lock and start transaction
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| StudFinderError::DatabaseLockFailed {
                operation: "add piece".to_string(),
            })?;

        let tx = conn.transaction().map_err(|e| StudFinderError::Database {
            operation: "begin transaction".to_string(),
            source: Box::new(e),
        })?;

        let existing = {
            let mut stmt = tx
                .prepare(
                    "SELECT id, part_number, color, category, quantity, confidence
                 FROM pieces WHERE id = ?",
                )
                .map_err(|e| StudFinderError::Database {
                    operation: "prepare select statement".to_string(),
                    source: Box::new(e),
                })?;

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
            .map_err(|e| StudFinderError::Database {
                operation: "query existing piece".to_string(),
                source: Box::new(e),
            })?
        };

        if let Some(existing_piece) = existing {
            debug!("Found existing piece, updating quantity");
            tx.execute(
                "UPDATE pieces SET quantity = ?1 WHERE id = ?2",
                params![
                    piece.quantity + existing_piece.quantity,
                    piece.id
                ],
            )
            .map_err(|e| StudFinderError::Database {
                operation: "update piece quantity".to_string(),
                source: Box::new(e),
            })?;
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
            .map_err(|e| StudFinderError::Database {
                operation: "insert new piece".to_string(),
                source: Box::new(e),
            })?;
        }

        tx.commit().map_err(|e| StudFinderError::Database {
            operation: "commit transaction".to_string(),
            source: Box::new(e),
        })?;
        debug!("Successfully added/updated piece in database");

        Ok(())
    }

    /// Retrieves a piece from the database by its ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the piece to retrieve
    ///
    /// # Returns
    ///
    /// `Ok(Some(Piece))` if the piece was found, `Ok(None)` if not found, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to prepare or execute the query
    pub fn get_piece(&self, id: &str) -> Result<Option<Piece>> {
        debug!("Fetching piece with id: {}", id);

        let conn = self
            .conn
            .lock()
            .map_err(|_| StudFinderError::DatabaseLockFailed {
                operation: "get piece".to_string(),
            })?;

        let mut stmt = conn
            .prepare(
                "SELECT id, part_number, color, category, quantity, confidence
             FROM pieces WHERE id = ?",
            )
            .map_err(|e| StudFinderError::Database {
                operation: "prepare select statement".to_string(),
                source: Box::new(e),
            })?;

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
            .map_err(|e| StudFinderError::Database {
                operation: format!("query piece with id '{}'", id),
                source: Box::new(e),
            })?;

        debug!("Piece lookup result: {:?}", piece);
        Ok(piece)
    }

    /// Lists all pieces in the inventory
    ///
    /// # Returns
    ///
    /// A vector of all pieces in the database, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to prepare or execute the query
    /// - Failed to collect the results
    pub fn list_pieces(&self) -> Result<Vec<Piece>> {
        debug!("Listing all pieces in inventory");

        let conn = self
            .conn
            .lock()
            .map_err(|_| StudFinderError::DatabaseLockFailed {
                operation: "list pieces".to_string(),
            })?;

        let mut stmt = conn
            .prepare("SELECT id, part_number, color, category, quantity, confidence FROM pieces")
            .map_err(|e| StudFinderError::Database {
                operation: "prepare select statement".to_string(),
                source: Box::new(e),
            })?;

        let pieces_result = stmt
            .query_map([], |row| {
                Ok(Piece {
                    id: row.get(0)?,
                    part_number: row.get(1)?,
                    color: row.get(2)?,
                    category: row.get(3)?,
                    quantity: row.get(4)?,
                    confidence: row.get(5)?,
                })
            })
            .map_err(|e| StudFinderError::Database {
                operation: "query all pieces".to_string(),
                source: Box::new(e),
            })?;

        let mut pieces = Vec::new();
        for piece_result in pieces_result {
            match piece_result {
                Ok(piece) => pieces.push(piece),
                Err(e) => {
                    warn!("Error processing piece row: {}", e);
                    return Err(StudFinderError::Database {
                        operation: "process piece row".to_string(),
                        source: Box::new(e),
                    });
                }
            }
        }

        debug!("Found {} pieces in inventory", pieces.len());
        Ok(pieces)
    }

    /// Updates the quantity of a piece in the database
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the piece to update
    /// * `quantity` - The new quantity to set
    ///
    /// # Returns
    ///
    /// `Ok(())` if the piece was updated successfully, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to execute the update
    /// - The piece with the specified ID does not exist
    pub fn update_quantity(&self, id: &str, quantity: i32) -> Result<()> {
        debug!("Updating quantity for piece {}: {}", id, quantity);

        let conn = self
            .conn
            .lock()
            .map_err(|_| StudFinderError::DatabaseLockFailed {
                operation: "update quantity".to_string(),
            })?;

        let rows_affected = conn
            .execute(
                "UPDATE pieces SET quantity = ?1 WHERE id = ?2",
                params![quantity, id],
            )
            .map_err(|e| StudFinderError::Database {
                operation: format!("update quantity for piece '{}'", id),
                source: Box::new(e),
            })?;

        if rows_affected == 0 {
            return Err(StudFinderError::PieceNotFound(id.to_string()));
        }

        Ok(())
    }

    /// Deletes a piece from the database
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the piece to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` if the piece was deleted successfully, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to execute the delete
    /// - The piece with the specified ID does not exist
    pub fn delete_piece(&self, id: &str) -> Result<()> {
        debug!("Deleting piece with id: {}", id);

        let conn = self
            .conn
            .lock()
            .map_err(|_| StudFinderError::DatabaseLockFailed {
                operation: "delete piece".to_string(),
            })?;

        let rows_affected = conn
            .execute("DELETE FROM pieces WHERE id = ?", [id])
            .map_err(|e| StudFinderError::Database {
                operation: format!("delete piece '{}'", id),
                source: Box::new(e),
            })?;

        if rows_affected == 0 {
            return Err(StudFinderError::PieceNotFound(id.to_string()));
        }

        Ok(())
    }

    /// Gets the current schema version
    ///
    /// # Returns
    ///
    /// The current schema version number, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to acquire the database lock
    /// - Failed to query the schema version
    fn get_schema_version(&self) -> Result<i32> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| StudFinderError::DatabaseLockFailed {
                operation: "get schema version".to_string(),
            })?;

        let version: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| StudFinderError::Database {
                operation: "query schema version".to_string(),
                source: Box::new(e),
            })?;

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

    #[test]
    fn test_piece_not_found() {
        let db = Database::new(":memory:").unwrap();
        db.init().unwrap();

        // Test get with non-existent ID
        let result = db.get_piece("non-existent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Test update with non-existent ID
        let result = db.update_quantity("non-existent", 5);
        assert!(result.is_err());
        match result.unwrap_err() {
            StudFinderError::PieceNotFound(id) => assert_eq!(id, "non-existent"),
            e => panic!("Expected PieceNotFound error, got: {:?}", e),
        }

        // Test delete with non-existent ID
        let result = db.delete_piece("non-existent");
        assert!(result.is_err());
        match result.unwrap_err() {
            StudFinderError::PieceNotFound(id) => assert_eq!(id, "non-existent"),
            e => panic!("Expected PieceNotFound error, got: {:?}", e),
        }
    }

    #[test]
    fn test_database_error_context() {
        // Test connection error with context
        let result = Database::new("/nonexistent/path/that/does/not/exist/db.sqlite");
        assert!(result.is_err());
        match result.unwrap_err() {
            StudFinderError::Database {
                operation,
                ..
            } => {
                assert_eq!(operation, "open connection");
            }
            e => panic!("Expected Database error, got: {:?}", e),
        }
    }
}
