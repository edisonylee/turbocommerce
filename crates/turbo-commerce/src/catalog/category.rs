//! Category types for product organization.

use crate::ids::CategoryId;
use serde::{Deserialize, Serialize};

/// A product category in the catalog hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    /// Unique category identifier.
    pub id: CategoryId,
    /// Parent category ID (None for root categories).
    pub parent_id: Option<CategoryId>,
    /// Category name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// Category description.
    pub description: Option<String>,
    /// Category image URL.
    pub image_url: Option<String>,
    /// Sort order position within parent.
    pub position: i32,
    /// Depth in the hierarchy (0 = root).
    pub level: i32,
    /// Materialized path for efficient tree queries (e.g., "1/5/12").
    pub path: String,
    /// Number of products in this category.
    pub product_count: i64,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
}

impl Category {
    /// Create a new root category.
    pub fn new_root(name: impl Into<String>, slug: impl Into<String>) -> Self {
        let id = CategoryId::generate();
        let now = current_timestamp();
        Self {
            id: id.clone(),
            parent_id: None,
            name: name.into(),
            slug: slug.into(),
            description: None,
            image_url: None,
            position: 0,
            level: 0,
            path: id.as_str().to_string(),
            product_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new child category.
    pub fn new_child(
        parent: &Category,
        name: impl Into<String>,
        slug: impl Into<String>,
    ) -> Self {
        let id = CategoryId::generate();
        let now = current_timestamp();
        Self {
            id: id.clone(),
            parent_id: Some(parent.id.clone()),
            name: name.into(),
            slug: slug.into(),
            description: None,
            image_url: None,
            position: 0,
            level: parent.level + 1,
            path: format!("{}/{}", parent.path, id.as_str()),
            product_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this is a root category.
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Check if this category is an ancestor of another.
    pub fn is_ancestor_of(&self, other: &Category) -> bool {
        other.path.starts_with(&self.path) && other.id != self.id
    }

    /// Check if this category is a descendant of another.
    pub fn is_descendant_of(&self, other: &Category) -> bool {
        self.path.starts_with(&other.path) && self.id != other.id
    }

    /// Get the ancestor IDs from the path.
    pub fn ancestor_ids(&self) -> Vec<CategoryId> {
        self.path
            .split('/')
            .filter(|s| !s.is_empty() && *s != self.id.as_str())
            .map(CategoryId::new)
            .collect()
    }

    /// Get the full path as category IDs.
    pub fn path_ids(&self) -> Vec<CategoryId> {
        self.path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(CategoryId::new)
            .collect()
    }

    /// Get the breadcrumb depth.
    pub fn depth(&self) -> usize {
        self.path.matches('/').count()
    }
}

/// Get current Unix timestamp.
fn current_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_category() {
        let cat = Category::new_root("Electronics", "electronics");
        assert!(cat.is_root());
        assert_eq!(cat.level, 0);
        assert_eq!(cat.name, "Electronics");
    }

    #[test]
    fn test_child_category() {
        let parent = Category::new_root("Electronics", "electronics");
        let child = Category::new_child(&parent, "Phones", "phones");

        assert!(!child.is_root());
        assert_eq!(child.level, 1);
        assert!(child.path.contains(&parent.id.as_str().to_string()));
    }

    #[test]
    fn test_hierarchy() {
        let root = Category::new_root("Root", "root");
        let child = Category::new_child(&root, "Child", "child");
        let grandchild = Category::new_child(&child, "Grandchild", "grandchild");

        assert!(root.is_ancestor_of(&child));
        assert!(root.is_ancestor_of(&grandchild));
        assert!(child.is_ancestor_of(&grandchild));

        assert!(child.is_descendant_of(&root));
        assert!(grandchild.is_descendant_of(&root));
        assert!(grandchild.is_descendant_of(&child));

        assert!(!root.is_ancestor_of(&root));
        assert!(!child.is_ancestor_of(&root));
    }
}
