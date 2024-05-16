use crate::dto::{RouteKindDTO, ServerDesc};
use htmlescape::encode_minimal;

pub fn create_route_list_html(server_desc: &ServerDesc) -> String {
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
    for x in server_desc.routes.iter() {
        let mut item = String::from("<tr>");
        let kind = match &x.kind {
            RouteKindDTO::Html { .. } => String::from("RouteKind::Html"),
            RouteKindDTO::Json { .. } => String::from("RouteKind::Json"),
            RouteKindDTO::Raw { .. } => String::from("RouteKind::Raw"),
            RouteKindDTO::Sse { .. } => String::from("RouteKind::Sse"),
            RouteKindDTO::Proxy { proxy } => {
                format!("RouteKind::Proxy('{}')", proxy)
            }
            RouteKindDTO::Dir { dir } => {
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
    let json = serde_json::to_string_pretty(&server_desc).expect("serde");
    markup.push_str(&encode_minimal(&json));
    markup.push_str("</pre></code>");
    wrapper.replace("{{route_list}}", markup.as_str())
}
