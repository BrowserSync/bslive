use bsnext_input::route::{DirRoute, ProxyRoute, Route, RouteKind};
use htmlescape::encode_minimal;

pub fn create_route_list_html(routes: &[Route]) -> String {
    let wrapper = include_str!("not_found.html");
    let mut markup = String::from("<table>");
    markup.push_str(
        r##"
    <thead>
        <tr>
            <td>Path</td>
            <td>Kind</td>
        </tr>
    </thead>
    "##,
    );
    for x in routes.iter() {
        let mut item = String::from("<tr>");
        let kind = match &x.kind {
            RouteKind::Html { .. } => String::from("RouteKind::Html"),
            RouteKind::Json { .. } => String::from("RouteKind::Json"),
            RouteKind::Raw { .. } => String::from("RouteKind::Raw"),
            RouteKind::Sse { .. } => String::from("RouteKind::Sse"),
            RouteKind::Proxy(ProxyRoute { proxy }) => {
                format!("RouteKind::Proxy('{}')", proxy)
            }
            RouteKind::Dir(DirRoute { dir }) => {
                format!("RouteKind::Dir('{}')", dir.clone())
            }
        };
        item.push_str(
            format!(
                "<td><a href='{}'><code>{}</code></a></td>\
                \
                <td><small>{}</small></td>",
                x.path, x.path, kind
            )
            .as_str(),
        );
        item.push_str("</tr>");
        markup.push_str(item.as_str());
    }
    markup.push_str("</table>");
    markup.push_str("<pre><code>");
    let json = serde_json::to_string_pretty(routes).expect("serde");
    markup.push_str(&encode_minimal(&json));
    markup.push_str("</pre></code>");
    wrapper.replace("{{route_list}}", markup.as_str())
}
