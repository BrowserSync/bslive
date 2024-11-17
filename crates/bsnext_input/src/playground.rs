use crate::path_def::PathDef;
use crate::route::{FallbackRoute, Opts, Route, RouteKind};
use bsnext_resp::builtin_strings::{BuiltinStringDef, BuiltinStrings};
use bsnext_resp::inject_addition::{AdditionPosition, InjectAddition};
use bsnext_resp::inject_opts::{InjectOpts, Injection, InjectionItem, MatcherList};
use bsnext_resp::path_matcher::PathMatcher;

#[derive(Debug, PartialEq, Default, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct Playground {
    pub html: String,
    pub js: Option<String>,
    pub css: Option<String>,
}

const FALLBACK_HTML: &str = "This is a BSLIVE playground";

impl Playground {
    pub fn as_routes(&self) -> Vec<Route> {
        let home_path = PathDef::try_new("/");
        let js_path = PathDef::try_new("/__bslive_playground.js");
        let css_path = PathDef::try_new("/__bslive_playground.css");
        let reset_path = PathDef::try_new("/reset.css");

        let (Ok(home), Ok(js), Ok(css), Ok(reset_path)) =
            (home_path, js_path, css_path, reset_path)
        else {
            return vec![];
        };

        let home_route = Route {
            path: home,
            kind: RouteKind::new_html(self.html.clone()),
            opts: Opts {
                cors: None,
                delay: None,
                watch: Default::default(),
                inject: playground_wrap(),
                headers: None,
                compression: None,
                ..Default::default()
            },
            fallback: Some(FallbackRoute {
                kind: RouteKind::new_html(FALLBACK_HTML),
                opts: Default::default(),
            }),
        };
        let js_route = Route {
            path: js,
            kind: RouteKind::new_raw(
                self.js
                    .as_ref()
                    .unwrap_or(&"// playground js is absent".to_string()),
            ),
            ..Default::default()
        };
        let css_route = Route {
            path: css,
            kind: RouteKind::new_raw(
                self.css
                    .as_ref()
                    .unwrap_or(&"/* playground css is absent */".to_string()),
            ),
            ..Default::default()
        };
        let reset_route = Route {
            path: reset_path,
            kind: RouteKind::new_raw(include_str!("../../../ui/styles/reset.css")),
            ..Default::default()
        };
        vec![home_route, js_route, css_route, reset_route]
    }
}

fn playground_wrap() -> InjectOpts {
    let prepend = r#"
<!doctype html>
  <html lang="en">
  <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1">
      <title>Document</title>
      <link rel="stylesheet" href="/__bslive_playground.css">
  </head>
  <body>
"#;
    let append = r#"
  <script src="/__bslive_playground.js"></script>
  </body>
</html>
"#;
    InjectOpts::Items(vec![
        InjectionItem {
            inner: Injection::Addition(InjectAddition {
                addition_position: AdditionPosition::Prepend(prepend.to_string()),
            }),
            only: Some(MatcherList::Item(PathMatcher::Str("/".to_string()))),
        },
        InjectionItem {
            inner: Injection::Addition(InjectAddition {
                addition_position: AdditionPosition::Append(append.to_string()),
            }),
            only: Some(MatcherList::Item(PathMatcher::Str("/".to_string()))),
        },
        InjectionItem {
            inner: Injection::BsLive(BuiltinStringDef {
                name: BuiltinStrings::Connector,
            }),
            only: Some(MatcherList::Item(PathMatcher::Str("/".to_string()))),
        },
    ])
}
