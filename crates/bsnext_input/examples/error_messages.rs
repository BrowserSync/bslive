use bsnext_input::route::Route;
use miette::{JSONReportHandler, NamedSource};
use serde_yaml::Value;
use std::io::Write;

fn main() -> miette::Result<()> {
    use miette::{Diagnostic, SourceSpan};
    use thiserror::Error;
    let input = include_str!("../../../bslive.yml");

    #[derive(Error, Diagnostic, Debug)]
    pub enum InputError {
        #[error(transparent)]
        #[diagnostic(transparent)]
        YamlParsing(#[from] YamlParsingError),
        #[error(transparent)]
        #[diagnostic(transparent)]
        BsLiveRules(#[from] BsLiveRulesError),
    }

    #[derive(Diagnostic, Debug, Error)]
    #[error("Invalid YAML input: {other_error}")]
    #[diagnostic()]
    pub struct YamlParsingError {
        // Note: label but no source code
        #[label = "{message}"]
        err_span: SourceSpan,
        #[source_code]
        src: NamedSource<String>,
        message: String,
        other_error: String,
    }

    #[derive(Diagnostic, Debug, Error)]
    #[error("bslive rules violated")]
    #[diagnostic()]
    pub struct BsLiveRulesError {
        // Note: label but no source code
        #[label = "{message}"]
        err_span: SourceSpan,
        #[source_code]
        src: NamedSource<String>,
        message: String,
        #[help]
        summary: Option<String>,
    }

    fn do_something(input: &str) -> miette::Result<()> {
        let as_input = serde_yaml::from_str::<bsnext_input::Input>(input);

        let Err(err) = as_input else { return Ok(()) };

        match (
            err.location(),
            serde_yaml::from_str::<serde_yaml::Value>(input),
        ) {
            (Some(loc), Ok(value)) => {
                let summary = routes_summary(&value);
                Err(InputError::BsLiveRules(BsLiveRulesError {
                    err_span: (loc.index()..loc.index() + 1).into(),
                    src: NamedSource::new(
                        "/Users/shaneosbourne/WebstormProjects/bslive/bslive.yml",
                        input.to_string(),
                    ),
                    message: err.to_string(),
                    // advise: "the YAML was formatted correctly, but we couldn't convert it into a valid input.".into(),
                    summary: summary.map(|s| s.as_summary()),
                })
                .into())
            }
            // in this case, the parsing was invalid
            (Some(loc), Err(err_2)) => Err(InputError::YamlParsing(YamlParsingError {
                err_span: (loc.index()..loc.index() + 1).into(),
                src: NamedSource::new(
                    "/Users/shaneosbourne/WebstormProjects/bslive/bslive.yml",
                    input.to_string(),
                ),
                message: err.to_string(),
                other_error: err_2.to_string(),
            })
            .into()),
            (None, _) => {
                unreachable!("if location is missing, wtf")
            }
        }
    }

    fn fmt_report(diag: miette::Report) -> String {
        let mut out = String::new();
        JSONReportHandler::new()
            .render_report(&mut out, diag.as_ref())
            .unwrap();
        out
    }

    fn json_report(input: &str) {
        match do_something(input) {
            Ok(_) => unreachable!(),
            Err(report) => {
                let json = fmt_report(report.into());
                println!("{}", json);
            }
        }
    }

    // json_report(input);
    // do_something(input)
    // fn other() -> miette::Result<()> {
    //     let exists = fs::exists("abc.def").into_diagnostic()?;
    //     if !exists {
    //         return Err(InputError::MissingFile {
    //             path: "abc.def".to_string(),
    //             advise: "ha".to_string(),
    //         }
    //         .into());
    //     }
    //     Ok(())
    // }
    let r = do_something(input);
    match r {
        Ok(_) => {}
        Err(err) => {
            let n = miette::GraphicalReportHandler::new();
            let mut inner = String::new();
            n.render_report(&mut inner, err.as_ref()).expect("write?");
            println!("{inner}")
        }
    }
    Ok(())
}

#[derive(Debug)]
enum RoutesSummary {
    OneValid(ValidRoute),
    AllValid(Vec<ValidRoute>),

    OneInvalid(InvalidRoute),
    AllInvalid(Vec<InvalidRoute>),

    SomeInvalid {
        valid: Vec<ValidRoute>,
        invalid: Vec<InvalidRoute>,
    },
}

impl RoutesSummary {
    fn as_summary(&self) -> String {
        fn int_to_ordinal_string(n: usize) -> String {
            let suffix = match n % 10 {
                1 if n % 100 != 11 => "st",
                2 if n % 100 != 12 => "nd",
                3 if n % 100 != 13 => "rd",
                _ => "th",
            };
            format!("{}{}", n, suffix)
        }

        match self {
            RoutesSummary::OneValid(_) => String::from("✅ valid route"),
            RoutesSummary::AllValid(all) => format!("✅ {} valid routes", all.len()),
            RoutesSummary::OneInvalid(_) => String::from("❌ invalid route"),
            RoutesSummary::AllInvalid(all) => format!("❌ {} invalid routes", all.len()),
            RoutesSummary::SomeInvalid { invalid, .. } if invalid.len() == 1 => {
                let Some(InvalidRoute(index, ..)) = invalid.first() else {
                    unreachable!("how?")
                };
                format!(
                    "{} route was invalid, the others were ok",
                    int_to_ordinal_string(index + 1)
                )
            }
            RoutesSummary::SomeInvalid { valid, invalid } if invalid.len() == 2 => {
                let combined = valid.len() + invalid.len();
                let invalid_nums = invalid
                    .iter()
                    .map(|InvalidRoute(index, ..)| int_to_ordinal_string(index + 1))
                    .collect::<Vec<_>>()
                    .join(" and ");

                format!(
                    "from your {} routes, {} were invalid.",
                    combined, invalid_nums,
                )
            }
            RoutesSummary::SomeInvalid { valid, invalid } => {
                let invalid_nums = invalid
                    .iter()
                    .map(|InvalidRoute(index, ..)| int_to_ordinal_string(index + 1))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!(
                    "{} routes were invalid. {} other routes were ok\nThe first error is highlighted above",
                    invalid_nums,
                    valid.len(),
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ValidRoute;
#[derive(Debug, Clone)]
struct InvalidRoute(pub usize);

fn routes_summary(v: &Value) -> Option<RoutesSummary> {
    let routes = &v["servers"][0]["routes"];

    let Value::Sequence(routes) = routes else {
        return None;
    };

    if routes.is_empty() {
        return None;
    }

    let mut valid = vec![];
    let mut invalid = vec![];

    for (index, x) in routes.iter().enumerate() {
        match serde_yaml::from_value::<Route>((*x).clone()) {
            Ok(_) => valid.push(ValidRoute),
            Err(_) => invalid.push(InvalidRoute(index)),
        }
    }

    let summary = match (routes.len(), valid.len(), invalid.len()) {
        (1, 1, 0) => {
            // one valid
            RoutesSummary::OneValid((*valid.get(0).unwrap()).clone())
        }
        (1, 0, 1) => {
            // one invalid
            let err = (*invalid.get(0).unwrap()).clone();
            RoutesSummary::OneInvalid(err)
        }
        (l1, l2, _) if l1 == l2 => {
            // all valid
            RoutesSummary::AllValid(valid)
        }
        (l1, _, l3) if l1 == l3 => {
            // all invalid
            RoutesSummary::AllInvalid(invalid)
        }
        (_, _, _) => {
            // some invalid
            RoutesSummary::SomeInvalid { valid, invalid }
        }
    };

    Some(summary)
}

#[test]
fn test_routes_summary() {
    let input = include_str!("../../../bslive.yml");
    let result = routes_summary(&serde_yaml::from_str::<Value>(input).unwrap());
    dbg!(result);
}
