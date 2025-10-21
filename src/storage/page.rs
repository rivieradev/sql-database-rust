// Page-based storage
// Databases don't store individual rows - they store "pages" (fixed-size blocks)
// This is because:
// 1. Disks read/write in blocks (usually 4KB or 8KB)
// 2. It's more efficient to read/write multiple rows at once
// 3. Pages can be cached in memory for faster access

use super::Row;
use serde::{Deserialize, Serialize};

/// A page is a fixed-size block that stores multiple rows
/// This is a simplified version - real databases have complex page formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// The page ID (unique identifier)
    pub id: usize,
    /// The rows stored in this page
    /// Vec<T> is Rust's growable array (like ArrayList in Java)
    pub rows: Vec<Row>,
    /// Maximum number of rows per page (simplified - real DBs use byte size)
    pub max_rows: usize,
}

impl Page {
    /// Create a new page with a given ID
    pub fn new(id: usize, max_rows: usize) -> Self {
        Self {
            id,
            rows: Vec::new(),
            max_rows,
        }
    }

    /// Insert a row into the page
    /// Returns true if successful, false if page is full
    pub fn insert(&mut self, row: Row) -> bool {
        if self.is_full() {
            return false;
        }

        self.rows.push(row);
        true
    }

    /// Check if the page is full
    pub fn is_full(&self) -> bool {
        self.rows.len() >= self.max_rows
    }

    /// Get a row by index within this page
    pub fn get(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// Get a mutable reference to a row
    /// 'mut' allows modifying the row (used for UPDATE operations)
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Row> {
        self.rows.get_mut(index)
    }

    /// Delete a row by index
    /// Returns the deleted row if successful
    pub fn delete(&mut self, index: usize) -> Option<Row> {
        if index < self.rows.len() {
            Some(self.rows.remove(index))
        } else {
            None
        }
    }

    /// Get the number of rows in this page
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if the page is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Page Manager - handles multiple pages
/// In a real database, this would also handle:
/// - Writing pages to disk
/// - Caching frequently accessed pages
/// - Managing free space
#[derive(Debug)]
pub struct PageManager {
    /// All pages in memory (in a real DB, these would be on disk)
    pages: Vec<Page>,
    /// Maximum rows per page
    max_rows_per_page: usize,
}

impl PageManager {
    /// Create a new page manager
    pub fn new(max_rows_per_page: usize) -> Self {
        Self {
            pages: Vec::new(),
            max_rows_per_page,
        }
    }

    /// Insert a row, creating new pages as needed
    /// Returns (page_id, row_index_in_page)
    pub fn insert(&mut self, row: Row) -> (usize, usize) {
        // Try to find a page with space
        for page in &mut self.pages {
            if !page.is_full() {
                let row_index = page.rows.len();
                page.insert(row);
                return (page.id, row_index);
            }
        }

        // No space found - create a new page
        let page_id = self.pages.len();
        let mut new_page = Page::new(page_id, self.max_rows_per_page);
        new_page.insert(row);
        self.pages.push(new_page);

        (page_id, 0)
    }

    /// Get a row by global row ID
    /// Row ID format: page_id * max_rows_per_page + row_index
    pub fn get(&self, row_id: usize) -> Option<&Row> {
        let page_id = row_id / self.max_rows_per_page;
        let row_index = row_id % self.max_rows_per_page;

        self.pages.get(page_id)?.get(row_index)
    }

    /// Get a mutable reference to a row
    pub fn get_mut(&mut self, row_id: usize) -> Option<&mut Row> {
        let page_id = row_id / self.max_rows_per_page;
        let row_index = row_id % self.max_rows_per_page;

        self.pages.get_mut(page_id)?.get_mut(row_index)
    }

    /// Get all rows (for table scans)
    /// Returns an iterator over all rows with their row IDs
    pub fn scan(&self) -> Vec<(usize, &Row)> {
        let mut results = Vec::new();

        for page in &self.pages {
            for (row_index, row) in page.rows.iter().enumerate() {
                let row_id = page.id * self.max_rows_per_page + row_index;
                results.push((row_id, row));
            }
        }

        results
    }

    /// Get the total number of rows across all pages
    pub fn total_rows(&self) -> usize {
        self.pages.iter().map(|p| p.len()).sum()
    }
}
