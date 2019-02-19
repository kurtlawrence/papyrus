use reqwest;

/// The result status of the version query.
#[derive(Debug, PartialEq)]
pub enum Status {
    /// The current version is the same as the latest on `crates.io`. The inner data is the current version `semver`.
    UpToDate(String),
    /// The current version is not the same as the latest on `crates.io`. The inner data is the current version (on `crates.io`) `semver`.
    OutOfDate(String),
}

/// Queries `crates.io` to get the latest `semver` of `papyrus`.
pub fn query() -> Result<Status, ()> {
    let text = web_req()?;
    let current = env!("CARGO_PKG_VERSION");
    parse(&text, current)
}

fn parse(text: &str, current: &str) -> Result<Status, ()> {
    // parse to get the version number
    let ver = match text.split('\"').skip_while(|&x| x != "max_version").nth(2) {
        // json format ("max_version":"#.#.#") hence will parse as [max_version, :, #,#,#]
        Some(ver) => Ok(ver),
        None => Err(()),
    }?;

    if ver == current {
        Ok(Status::UpToDate(ver.to_string()))
    } else {
        Ok(Status::OutOfDate(ver.to_string()))
    }
}

fn web_req() -> Result<String, ()> {
    match reqwest::get("https://crates.io/api/v1/crates/papyrus") {
        Ok(mut response) => match response.text() {
            Ok(text) => Ok(text),
            Err(_) => Err(()),
        },
        Err(_) => Err(()),
    }
}

#[test]
fn parse_test() {
    let current = "0.4.2";
    let ver_str = r#""max_version":"0.4.2""#;
    assert_eq!(
        parse(ver_str, current),
        Ok(Status::UpToDate(current.to_string()))
    );
    let ver_str = r#""max_version":"0.4.3""#;
    assert_eq!(
        parse(ver_str, current),
        Ok(Status::OutOfDate("0.4.3".to_string()))
    );
    let ver_str = r#""max_versions":"0.4.3""#;
    assert_eq!(parse(ver_str, current), Err(()));
    let ver_str = r#""max_version":"#;
    assert_eq!(parse(ver_str, current), Err(()));
}

#[test]
fn test_web_req() {
    // verify that the return crate is the right one!
    let req = web_req();
    match req {
        Err(_) => panic!("failed to query crates.io"),
        Ok(text) => {
            assert!(text.starts_with(r#"{"crate":{"id":"papyrus","name":"papyrus","#));
        }
    }
    // test the general query
    let req = query();
    match req {
        Ok(s) => println!("{:?}", s),
        Err(_) => println!("errored",),
    }
}
