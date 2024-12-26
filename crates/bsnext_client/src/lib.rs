pub const DIR: &str = "dist";
pub const UI_CSS: &str = include_str!("../../../ui/dist/index.css");
pub const UI_JS: &str = include_str!("../../../ui/dist/index.js");
const UI_HTML: &str = include_str!("../../../ui/index.html");
pub const INJECT_JS: &str = include_str!("../../../inject/dist/index.js");
pub const REPLACE_STR: &str = "window.$BSLIVE_INJECT_CONFIG$";
pub const WS_PATH: &str = "/__bs_ws";

pub fn html_with_base(base_override: &str) -> String {
    let base = UI_HTML;
    let next = format!("<base href=\"{}\" />", base_override);
    let replaced = base.replace("<base href=\"/\" />", next.as_str());
    replaced
}

pub fn inject_js_with_config(inject: bsnext_dto::InjectConfig) -> String {
    let json = serde_json::to_string(&inject).expect("its a known type");
    INJECT_JS.replace(REPLACE_STR, &json)
}
