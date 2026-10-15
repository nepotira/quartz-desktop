//! # Filesystem Transaction & Undo/Redo Subsystem (`Cmd+Z` / `Shift+Cmd+Z`)
//!
//! Maintains an atomic journal of file management operations (Move, Copy, Rename,
//! Trash, Create Folder, Batch Rename), allowing safe reversible rollbacks.

use std::path::PathBuf;
use thiserror::Error;

/// Errors occurring during Undo or Redo filesystem rollbacks.
#[derive(Debug, Error, PartialEq)]
pub enum UndoError {
    /// Undo stack is empty.
    #[error("No actions available to undo")]
    NothingToUndo,
    /// Redo stack is empty.
    #[error("No actions available to redo")]
    NothingToRedo,
    /// Target file already exists at rollback location.
    #[error("Destination file already exists during rollback: {0}")]
    DestinationExists(PathBuf),
    /// Expected source file was missing during rollback.
    #[error("Source file missing during rollback: {0}")]
    SourceNotFound(PathBuf),
}

/// A reversible atomic filesystem operation.
#[derive(Debug, Clone, PartialEq)]
pub enum FsOperation {
    /// Moved file from source to destination.
    Move {
        /// Original location before move.
        source: PathBuf,
        /// New location after move.
        destination: PathBuf,
    },
    /// Copied file or directory.
    Copy {
        /// Source item that was cloned.
        source: PathBuf,
        /// New duplicate file created.
        created_destination: PathBuf,
    },
    /// Renamed an item.
    Rename {
        /// Old file path.
        old_path: PathBuf,
        /// New file path.
        new_path: PathBuf,
    },
    /// Moved an item to the system Trash.
    Trash {
        /// Original path before trashing.
        original_path: PathBuf,
        /// Path inside ~/.Trash.
        trash_path: PathBuf,
    },
    /// Restored an item from Trash back to its original path.
    RestoreFromTrash {
        /// Path inside ~/.Trash.
        trash_path: PathBuf,
        /// Restored original path.
        original_path: PathBuf,
    },
    /// Created a new folder.
    CreateFolder {
        /// Path to newly created directory.
        folder_path: PathBuf,
    },
    /// Batch renamed multiple files simultaneously.
    BatchRename {
        /// Pairs of (old_path, new_path) for each renamed item.
        changes: Vec<(PathBuf, PathBuf)>,
    },
}

impl FsOperation {
    /// Human-readable title of operation (e.g. "Undo Move of 'report.pdf'").
    pub fn description(&self) -> String {
        match self {
            Self::Move { source, .. } => {
                let name = source.file_name().and_then(|f| f.to_str()).unwrap_or("Item");
                format!("Move of '{name}'")
            }
            Self::Copy { source, .. } => {
                let name = source.file_name().and_then(|f| f.to_str()).unwrap_or("Item");
                format!("Copy of '{name}'")
            }
            Self::Rename { old_path, .. } => {
                let name = old_path.file_name().and_then(|f| f.to_str()).unwrap_or("Item");
                format!("Rename of '{name}'")
            }
            Self::Trash { original_path, .. } => {
                let name = original_path.file_name().and_then(|f| f.to_str()).unwrap_or("Item");
                format!("Move '{name}' to Trash")
            }
            Self::RestoreFromTrash { original_path, .. } => {
                let name = original_path.file_name().and_then(|f| f.to_str()).unwrap_or("Item");
                format!("Put Back '{name}'")
            }
            Self::CreateFolder { folder_path } => {
                let name = folder_path.file_name().and_then(|f| f.to_str()).unwrap_or("Folder");
                format!("New Folder '{name}'")
            }
            Self::BatchRename { changes } => {
                format!("Batch Rename of {} items", changes.len())
            }
        }
    }

    /// Inverts the operation to create its inverse compensating action.
    pub fn invert(&self) -> FsOperation {
        match self {
            Self::Move { source, destination } => Self::Move {
                source: destination.clone(),
                destination: source.clone(),
            },
            Self::Copy { source, created_destination } => Self::Move {
                source: created_destination.clone(),
                destination: source.clone(),
            },
            Self::Rename { old_path, new_path } => Self::Rename {
                old_path: new_path.clone(),
                new_path: old_path.clone(),
            },
            Self::Trash { original_path, trash_path } => Self::RestoreFromTrash {
                trash_path: trash_path.clone(),
                original_path: original_path.clone(),
            },
            Self::RestoreFromTrash { trash_path, original_path } => Self::Trash {
                original_path: original_path.clone(),
                trash_path: trash_path.clone(),
            },
            Self::CreateFolder { folder_path } => Self::Trash {
                original_path: folder_path.clone(),
                trash_path: folder_path.clone(),
            },
            Self::BatchRename { changes } => {
                let inverted_changes = changes
                    .iter()
                    .map(|(old, new)| (new.clone(), old.clone()))
                    .collect();
                Self::BatchRename {
                    changes: inverted_changes,
                }
            }
        }
    }
}

/// Transaction Journal managing the undo and redo history stacks.
pub struct FsTransactionJournal {
    undo_stack: Vec<FsOperation>,
    redo_stack: Vec<FsOperation>,
    max_history_depth: usize,
}

impl Default for FsTransactionJournal {
    fn default() -> Self {
        Self::new(100)
    }
}

impl FsTransactionJournal {
    /// Creates a new journal with specified history capacity.
    pub fn new(max_history_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history_depth: max_history_depth.max(10),
        }
    }

    /// Records an executed filesystem operation into the undo stack and clears redo stack.
    pub fn record_operation(&mut self, op: FsOperation) {
        if self.undo_stack.len() >= self.max_history_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(op);
        self.redo_stack.clear();
    }

    /// Whether an operation is available to undo.
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Description of the next undo action if any.
    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack.last().map(|op| format!("Undo {}", op.description()))
    }

    /// Whether an operation is available to redo.
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Description of the next redo action if any.
    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack.last().map(|op| format!("Redo {}", op.description()))
    }

    /// Pops the last operation to undo, returning its inverted compensating operation.
    pub fn prepare_undo(&mut self) -> Result<FsOperation, UndoError> {
        let op = self.undo_stack.pop().ok_or(UndoError::NothingToUndo)?;
        let inverted = op.invert();
        self.redo_stack.push(op);
        Ok(inverted)
    }

    /// Pops the last undone operation to redo, returning the forward operation.
    pub fn prepare_redo(&mut self) -> Result<FsOperation, UndoError> {
        let op = self.redo_stack.pop().ok_or(UndoError::NothingToRedo)?;
        self.undo_stack.push(op.clone());
        Ok(op)
    }

    /// Clears both undo and redo stacks.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_transaction_undo_redo_inversion() {
        let mut journal = FsTransactionJournal::default();

        let move_op = FsOperation::Move {
            source: PathBuf::from("/Users/demo/Documents/file.txt"),
            destination: PathBuf::from("/Users/demo/Desktop/file.txt"),
        };

        journal.record_operation(move_op);
        assert!(journal.can_undo());
        assert!(!journal.can_redo());
        assert_eq!(journal.undo_description(), Some("Undo Move of 'file.txt'".into()));

        // Perform Undo
        let undo_action = journal.prepare_undo().unwrap();
        assert_eq!(
            undo_action,
            FsOperation::Move {
                source: PathBuf::from("/Users/demo/Desktop/file.txt"),
                destination: PathBuf::from("/Users/demo/Documents/file.txt"),
            }
        );

        assert!(!journal.can_undo());
        assert!(journal.can_redo());

        // Perform Redo
        let redo_action = journal.prepare_redo().unwrap();
        assert_eq!(
            redo_action,
            FsOperation::Move {
                source: PathBuf::from("/Users/demo/Documents/file.txt"),
                destination: PathBuf::from("/Users/demo/Desktop/file.txt"),
            }
        );

        assert!(journal.can_undo());
        assert!(!journal.can_redo());
    }
}
