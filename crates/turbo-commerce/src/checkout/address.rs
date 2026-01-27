//! Address types.

use crate::ids::AddressId;
use serde::{Deserialize, Serialize};

/// A postal address.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Address {
    /// Address ID (None for unsaved addresses).
    pub id: Option<AddressId>,
    /// First name.
    pub first_name: String,
    /// Last name.
    pub last_name: String,
    /// Company name.
    pub company: Option<String>,
    /// Address line 1.
    pub address1: String,
    /// Address line 2 (apt, suite, etc.).
    pub address2: Option<String>,
    /// City.
    pub city: String,
    /// State/province name.
    pub province: Option<String>,
    /// State/province code (e.g., "CA").
    pub province_code: Option<String>,
    /// Country name.
    pub country: String,
    /// Country code (e.g., "US").
    pub country_code: String,
    /// Postal/ZIP code.
    pub zip: String,
    /// Phone number.
    pub phone: Option<String>,
}

impl Address {
    /// Create a new address.
    pub fn new(
        first_name: impl Into<String>,
        last_name: impl Into<String>,
        address1: impl Into<String>,
        city: impl Into<String>,
        country: impl Into<String>,
        country_code: impl Into<String>,
        zip: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            first_name: first_name.into(),
            last_name: last_name.into(),
            company: None,
            address1: address1.into(),
            address2: None,
            city: city.into(),
            province: None,
            province_code: None,
            country: country.into(),
            country_code: country_code.into(),
            zip: zip.into(),
            phone: None,
        }
    }

    /// Get full name.
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Format as single line.
    pub fn one_line(&self) -> String {
        let mut parts = vec![self.address1.clone()];
        if let Some(ref addr2) = self.address2 {
            parts.push(addr2.clone());
        }
        parts.push(self.city.clone());
        if let Some(ref province) = self.province_code {
            parts.push(province.clone());
        }
        parts.push(self.zip.clone());
        parts.push(self.country_code.clone());
        parts.join(", ")
    }

    /// Format as multi-line.
    pub fn multi_line(&self) -> String {
        let mut lines = vec![self.full_name()];
        if let Some(ref company) = self.company {
            lines.push(company.clone());
        }
        lines.push(self.address1.clone());
        if let Some(ref addr2) = self.address2 {
            lines.push(addr2.clone());
        }
        let city_line = if let Some(ref province) = self.province_code {
            format!("{}, {} {}", self.city, province, self.zip)
        } else {
            format!("{} {}", self.city, self.zip)
        };
        lines.push(city_line);
        lines.push(self.country.clone());
        lines.join("\n")
    }

    /// Check if address is complete.
    pub fn is_complete(&self) -> bool {
        !self.first_name.is_empty()
            && !self.last_name.is_empty()
            && !self.address1.is_empty()
            && !self.city.is_empty()
            && !self.country_code.is_empty()
            && !self.zip.is_empty()
    }
}

impl Default for Address {
    fn default() -> Self {
        Self {
            id: None,
            first_name: String::new(),
            last_name: String::new(),
            company: None,
            address1: String::new(),
            address2: None,
            city: String::new(),
            province: None,
            province_code: None,
            country: String::new(),
            country_code: String::new(),
            zip: String::new(),
            phone: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let addr = Address::new(
            "John",
            "Doe",
            "123 Main St",
            "San Francisco",
            "United States",
            "US",
            "94102",
        );
        assert_eq!(addr.full_name(), "John Doe");
        assert!(addr.is_complete());
    }

    #[test]
    fn test_address_formatting() {
        let mut addr = Address::new(
            "Jane",
            "Smith",
            "456 Oak Ave",
            "Los Angeles",
            "United States",
            "US",
            "90001",
        );
        addr.province_code = Some("CA".to_string());

        assert!(addr.one_line().contains("Los Angeles"));
        assert!(addr.one_line().contains("CA"));
    }
}
