use crate::route_effect::RouteEffect;
use axum::extract::{Query, Request};
use bsnext_guards::path_matcher::PathMatcher;
use bsnext_guards::MatcherList;
use bsnext_input::route::Route;
use bsnext_query::dynamic_query_params::{DynamicQueryParams, InjectParam};
use bsnext_resp::builtin_strings::{BuiltinStringDef, BuiltinStrings};
use bsnext_resp::inject_opts::{InjectOpts, Injection, InjectionItem};
use http::Uri;

#[derive(Debug, PartialEq)]
pub struct Injections {
    items: Vec<InjectionItem>,
}

impl RouteEffect for Injections {
    fn new_opt(route: &Route, _req: &Request, uri: &Uri, _outer_uri: &Uri) -> Option<Self> {
        let mut items = match &route.opts.inject {
            InjectOpts::Bool(true) => {
                vec![InjectionItem {
                    inner: Injection::BsLive(BuiltinStringDef {
                        name: BuiltinStrings::Connector,
                    }),
                    only: None,
                }]
            }
            InjectOpts::Bool(false) => {
                vec![]
            }
            InjectOpts::Items(items) if items.is_empty() => vec![],
            // todo: is this too expensive?
            InjectOpts::Items(items) => items.to_owned(),
            InjectOpts::Item(item) => vec![item.to_owned()],
        };
        if let Ok(Query(DynamicQueryParams {
            inject: Some(inject_append),
            ..
        })) = Query::try_from_uri(uri)
        {
            match inject_append {
                InjectParam::Other(other) if other == "false" => {
                    items = vec![];
                }
                InjectParam::BuiltinStrings(str) => items.push(InjectionItem {
                    inner: Injection::BsLive(BuiltinStringDef {
                        name: str.to_owned(),
                    }),
                    only: Some(MatcherList::Item(PathMatcher::pathname(uri.path()))),
                }),
                InjectParam::Other(_) => todo!("other?"),
            }
        }
        if items.is_empty() {
            return None;
        }
        Some(Injections { items })
    }
}

impl Injections {
    pub fn items(&self) -> Vec<InjectionItem> {
        self.items.clone()
    }
}
