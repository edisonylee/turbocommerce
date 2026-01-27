//! Review data models.

use serde::{Deserialize, Serialize};

/// Product reviews summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub product_id: String,
    pub average_rating: f32,
    pub total_reviews: u32,
    pub rating_distribution: RatingDistribution,
}

/// Distribution of ratings (1-5 stars).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingDistribution {
    pub five_star: u32,
    pub four_star: u32,
    pub three_star: u32,
    pub two_star: u32,
    pub one_star: u32,
}

impl RatingDistribution {
    /// Get percentage for a rating level.
    pub fn percentage(&self, stars: u8, total: u32) -> f32 {
        if total == 0 {
            return 0.0;
        }
        let count = match stars {
            5 => self.five_star,
            4 => self.four_star,
            3 => self.three_star,
            2 => self.two_star,
            1 => self.one_star,
            _ => 0,
        };
        (count as f32 / total as f32) * 100.0
    }
}

/// Individual review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: String,
    pub product_id: String,
    pub author: String,
    pub rating: u8,
    pub title: String,
    pub body: String,
    pub date: String,
    #[serde(default)]
    pub verified_purchase: bool,
    #[serde(default)]
    pub helpful_votes: u32,
}

impl Review {
    /// Render star rating as HTML.
    pub fn render_stars(&self) -> String {
        let filled = self.rating as usize;
        let empty = 5 - filled;
        format!(
            "{}{}",
            "★".repeat(filled),
            "☆".repeat(empty)
        )
    }
}

/// Reviews response from API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewsResponse {
    pub summary: ReviewSummary,
    pub reviews: Vec<Review>,
    #[serde(default)]
    pub has_more: bool,
}
