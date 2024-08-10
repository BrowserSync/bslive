use axum::routing::get;
use axum::Router;
use bsnext_resp::inject_opts::InjectionItem;

#[tokio::test]
async fn test_inject_all() -> anyhow::Result<()> {
    let inject_items: Vec<InjectionItem> = serde_yaml::from_str(
        r#"
- prepend: "<before>"
- append: "<after>"
- append: "<after2>"
    "#,
    )?;

    let router = Router::new().route("/", get(|| async { String::from("hey!") }));
    let actual = helpers::run(router, "/", &inject_items).await?;
    let expected = "<before>hey!<after><after2>";

    assert_eq!(&actual, expected);
    Ok(())
}
#[tokio::test]
async fn test_inject_only_1_level() -> anyhow::Result<()> {
    let items: Vec<InjectionItem> = serde_yaml::from_str(
        r#"
- prepend: --every--
- append: --lol--
  only:
    - /*.css
    "#,
    )?;

    let router = Router::new()
        .route("/", get(|| async { String::from("home page") }))
        .route("/abc.css", get(|| async { String::from("css content") }));

    let actual = helpers::run(router.clone(), "/abc.css", &items).await?;
    let expected = "--every--css content--lol--";
    assert_eq!(&actual, expected);

    let actual = helpers::run(router.clone(), "/", &items).await?;
    let expected = "--every--home page";
    assert_eq!(&actual, expected);

    Ok(())
}

mod helpers {
    use axum::body::Body;
    use axum::response::IntoResponse;
    use axum::{middleware, Extension, Router};
    use bsnext_resp::inject_opts::InjectionItem;
    use bsnext_resp::{response_modifications_layer, InjectHandling};
    use http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    pub async fn run(router: Router, uri: &str, items: &[InjectionItem]) -> anyhow::Result<String> {
        let router = router
            .layer(middleware::from_fn(response_modifications_layer))
            .layer(Extension(InjectHandling {
                items: items.to_vec(),
            }));

        let r = Request::get(uri).body(Body::empty())?;
        let output = router.oneshot(r).await?;
        let res = output
            .into_response()
            .into_body()
            .collect()
            .await?
            .to_bytes();

        let s = std::str::from_utf8(&res[..])?;
        Ok(s.to_string())
    }
}
