//! CMS data models for marketing content.

use serde::{Deserialize, Serialize};

/// Hero banner content with A/B testing support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroContent {
    pub headline: String,
    pub subheadline: String,
    pub cta_text: String,
    pub cta_url: String,
    pub background_image: Option<String>,
    pub variant: String,
}

impl Default for HeroContent {
    fn default() -> Self {
        Self {
            headline: "Transform Your Business".to_string(),
            subheadline: "The all-in-one platform for modern teams".to_string(),
            cta_text: "Get Started Free".to_string(),
            cta_url: "/signup".to_string(),
            background_image: None,
            variant: "control".to_string(),
        }
    }
}

impl HeroContent {
    /// Get variant A content (control).
    pub fn variant_a() -> Self {
        Self::default()
    }

    /// Get variant B content (challenger).
    pub fn variant_b() -> Self {
        Self {
            headline: "Scale Without Limits".to_string(),
            subheadline: "Join 10,000+ companies growing with us".to_string(),
            cta_text: "Start Your Free Trial".to_string(),
            cta_url: "/trial".to_string(),
            background_image: None,
            variant: "challenger".to_string(),
        }
    }
}

/// A feature block for the features section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub icon: String,
    pub title: String,
    pub description: String,
}

impl Feature {
    pub fn new(icon: &str, title: &str, description: &str) -> Self {
        Self {
            icon: icon.to_string(),
            title: title.to_string(),
            description: description.to_string(),
        }
    }
}

/// Features section content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesContent {
    pub section_title: String,
    pub section_subtitle: String,
    pub features: Vec<Feature>,
}

impl Default for FeaturesContent {
    fn default() -> Self {
        Self {
            section_title: "Everything you need".to_string(),
            section_subtitle: "Powerful features to help your team succeed".to_string(),
            features: vec![
                Feature::new(
                    "⚡",
                    "Lightning Fast",
                    "Edge-powered performance that scales globally with sub-millisecond latency.",
                ),
                Feature::new(
                    "🔒",
                    "Enterprise Security",
                    "SOC 2 compliant with end-to-end encryption and role-based access control.",
                ),
                Feature::new(
                    "📊",
                    "Real-time Analytics",
                    "Actionable insights with customizable dashboards and automated reports.",
                ),
                Feature::new(
                    "🔄",
                    "Seamless Integrations",
                    "Connect with 100+ tools your team already uses, or build custom integrations.",
                ),
                Feature::new(
                    "🤝",
                    "Team Collaboration",
                    "Real-time editing, comments, and workflows to keep everyone aligned.",
                ),
                Feature::new(
                    "📱",
                    "Mobile First",
                    "Native apps for iOS and Android with offline support and push notifications.",
                ),
            ],
        }
    }
}

/// A customer testimonial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Testimonial {
    pub quote: String,
    pub author_name: String,
    pub author_title: String,
    pub company: String,
    pub avatar_url: Option<String>,
}

impl Testimonial {
    pub fn new(quote: &str, name: &str, title: &str, company: &str) -> Self {
        Self {
            quote: quote.to_string(),
            author_name: name.to_string(),
            author_title: title.to_string(),
            company: company.to_string(),
            avatar_url: None,
        }
    }
}

/// Testimonials section content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestimonialsContent {
    pub section_title: String,
    pub testimonials: Vec<Testimonial>,
}

impl Default for TestimonialsContent {
    fn default() -> Self {
        Self {
            section_title: "Trusted by industry leaders".to_string(),
            testimonials: vec![
                Testimonial::new(
                    "This platform has transformed how our team works. We've seen a 40% increase in productivity since switching.",
                    "Sarah Chen",
                    "VP of Engineering",
                    "TechCorp",
                ),
                Testimonial::new(
                    "The best investment we've made this year. The ROI was visible within the first month.",
                    "Michael Rodriguez",
                    "CEO",
                    "StartupXYZ",
                ),
                Testimonial::new(
                    "Finally, a solution that actually delivers on its promises. Our customers love the improved experience.",
                    "Emily Watson",
                    "Head of Product",
                    "Enterprise Co",
                ),
            ],
        }
    }
}

/// Call-to-action section content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtaContent {
    pub headline: String,
    pub subheadline: String,
    pub primary_cta_text: String,
    pub primary_cta_url: String,
    pub secondary_cta_text: Option<String>,
    pub secondary_cta_url: Option<String>,
}

impl Default for CtaContent {
    fn default() -> Self {
        Self {
            headline: "Ready to get started?".to_string(),
            subheadline: "Join thousands of teams already using our platform.".to_string(),
            primary_cta_text: "Start Free Trial".to_string(),
            primary_cta_url: "/signup".to_string(),
            secondary_cta_text: Some("Schedule Demo".to_string()),
            secondary_cta_url: Some("/demo".to_string()),
        }
    }
}

/// Newsletter signup form state.
#[derive(Debug, Clone, Default)]
pub struct NewsletterState {
    pub email: String,
    pub subscribed: bool,
    pub error: Option<String>,
}

/// A/B test assignment for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperimentVariant {
    Control,
    Challenger,
}

impl ExperimentVariant {
    /// Determine variant from cookie or random assignment.
    pub fn from_cookie(cookie_value: Option<&str>) -> Self {
        match cookie_value {
            Some("B") | Some("challenger") => Self::Challenger,
            _ => Self::Control,
        }
    }

    /// Get variant name for analytics.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Challenger => "challenger",
        }
    }
}

/// Full page content loaded from CMS.
#[derive(Debug, Clone, Default)]
pub struct LandingPageContent {
    pub hero: HeroContent,
    pub features: FeaturesContent,
    pub testimonials: TestimonialsContent,
    pub cta: CtaContent,
}

impl LandingPageContent {
    /// Load page content for a given variant.
    pub fn for_variant(variant: ExperimentVariant) -> Self {
        let hero = match variant {
            ExperimentVariant::Control => HeroContent::variant_a(),
            ExperimentVariant::Challenger => HeroContent::variant_b(),
        };

        Self {
            hero,
            features: FeaturesContent::default(),
            testimonials: TestimonialsContent::default(),
            cta: CtaContent::default(),
        }
    }
}
