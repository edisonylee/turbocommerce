//! Shell template abstraction.

/// Head content for the shell.
#[derive(Debug, Clone, Default)]
pub struct HeadContent {
    /// Page title.
    pub title: Option<String>,
    /// Meta tags.
    pub meta: Vec<(String, String)>,
    /// Link tags (stylesheets, etc.).
    pub links: Vec<String>,
    /// Inline scripts in head.
    pub scripts: Vec<String>,
}

impl HeadContent {
    /// Create new head content with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..Default::default()
        }
    }

    /// Add a meta tag.
    pub fn with_meta(mut self, name: &str, content: &str) -> Self {
        self.meta.push((name.to_string(), content.to_string()));
        self
    }

    /// Add a stylesheet link.
    pub fn with_stylesheet(mut self, href: &str) -> Self {
        self.links
            .push(format!(r#"<link rel="stylesheet" href="{}">"#, href));
        self
    }

    /// Add inline CSS styles.
    pub fn with_style(mut self, css: &str) -> Self {
        self.links.push(format!("<style>{}</style>", css));
        self
    }

    /// Render head content to HTML.
    pub fn render(&self) -> String {
        let mut html = String::new();

        if let Some(title) = &self.title {
            html.push_str(&format!("<title>{}</title>\n", title));
        }

        for (name, content) in &self.meta {
            html.push_str(&format!(
                r#"<meta name="{}" content="{}">"#,
                name, content
            ));
            html.push('\n');
        }

        for link in &self.links {
            html.push_str(link);
            html.push('\n');
        }

        for script in &self.scripts {
            html.push_str(&format!("<script>{}</script>\n", script));
        }

        html
    }
}

/// Shell template with section placeholders.
#[derive(Debug, Clone)]
pub struct Shell {
    /// Include doctype declaration.
    pub doctype: bool,
    /// Head content.
    pub head: HeadContent,
    /// HTML before sections (opening body, wrapper divs, etc.).
    pub body_start: String,
    /// HTML after sections (closing tags).
    pub body_end: String,
}

impl Shell {
    /// Create a new shell with basic structure.
    pub fn new(head: HeadContent) -> Self {
        Self {
            doctype: true,
            head,
            body_start: "<body>\n<main>\n".to_string(),
            body_end: "</main>\n</body>\n</html>".to_string(),
        }
    }

    /// Set custom body start HTML.
    pub fn with_body_start(mut self, html: impl Into<String>) -> Self {
        self.body_start = html.into();
        self
    }

    /// Set custom body end HTML.
    pub fn with_body_end(mut self, html: impl Into<String>) -> Self {
        self.body_end = html.into();
        self
    }

    /// Render the opening part of the shell (before sections).
    pub fn render_opening(&self) -> String {
        let mut html = String::new();

        if self.doctype {
            html.push_str("<!DOCTYPE html>\n");
        }

        html.push_str("<html>\n<head>\n");
        html.push_str(&self.head.render());
        html.push_str("</head>\n");
        html.push_str(&self.body_start);

        html
    }

    /// Render the closing part of the shell (after sections).
    pub fn render_closing(&self) -> String {
        self.body_end.clone()
    }
}
