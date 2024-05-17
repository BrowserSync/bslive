pub const DIR: &str = "dist";
pub const UI_CSS: &str = include_str!("../ui/dist/index.css");
pub const UI_JS: &str = include_str!("../ui/dist/index.js");
const UI_HTML: &str = include_str!("../ui/index.html");

pub fn html_with_base(base_override: &str) -> String {
    let base = UI_HTML;
    let next = format!("<base href=\"{}\" />", base_override);
    let replaced = base.replace("<base href=\"/\" />", next.as_str());
    replaced
}
