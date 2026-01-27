//! Out-of-order streaming support.

/// Strategy for handling section ordering in the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderingStrategy {
    /// Stream sections in DOM order.
    /// May delay fast sections waiting for slow ones.
    #[default]
    Strict,

    /// Stream sections as they complete.
    /// Includes JavaScript to reorder in the browser.
    OutOfOrder,

    /// Stream sections as they complete.
    /// No reordering - sections are independent (e.g., islands).
    Independent,
}

impl OrderingStrategy {
    /// Check if this strategy allows out-of-order delivery.
    pub fn allows_out_of_order(&self) -> bool {
        !matches!(self, Self::Strict)
    }

    /// Check if this strategy needs reorder script.
    pub fn needs_reorder_script(&self) -> bool {
        matches!(self, Self::OutOfOrder)
    }
}

/// Generate JavaScript for reordering out-of-order sections.
///
/// This script moves sections to their correct DOM positions
/// after they arrive out of order.
pub fn generate_reorder_script(section_ids: &[&str]) -> String {
    let ids_json: Vec<String> = section_ids.iter().map(|s| format!("\"{}\"", s)).collect();

    format!(
        r#"<script>
(function() {{
  const order = [{}];
  const container = document.currentScript.parentElement;

  function reorder() {{
    const sections = {{}};
    container.querySelectorAll('[data-section]').forEach(el => {{
      sections[el.dataset.section] = el;
    }});

    order.forEach(id => {{
      if (sections[id]) {{
        container.appendChild(sections[id]);
      }}
    }});
  }}

  // Reorder when all sections are loaded
  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', reorder);
  }} else {{
    reorder();
  }}
}})();
</script>"#,
        ids_json.join(", ")
    )
}

/// Wrap section HTML with data attribute for reordering.
pub fn wrap_section_for_reorder(section_id: &str, html: &str) -> String {
    format!(
        r#"<div data-section="{}">{}</div>"#,
        section_id, html
    )
}
