use expectest::prelude::*;
use super::*;
use super::match_request;
use pact_matching::models::{Interaction, Request, Response, OptionalBody};
use pact_matching::Mismatch;

#[test]
fn match_request_returns_a_match_for_identical_requests() {
    let request = Request::default_request();
    let response = Response::default_response();
    let interaction = Interaction { description: s!("test"), provider_states: vec![],
        request: request.clone(), response: response.clone() };
    let interactions = vec![interaction.clone()];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_returns_a_not_found_for_no_interactions() {
    let request = Request::default_request();
    let interactions = vec![];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestNotFound(request)));
}

#[test]
fn match_request_returns_a_match_for_multiple_identical_requests() {
    let request = Request::default_request();
    let response = Response::default_response();
    let interaction = Interaction { description: s!("test"), provider_states: vec![],
        request: request.clone(), response: response.clone() };
    let interactions = vec![interaction.clone(),
        Interaction { description: s!("test2"), provider_states: vec![],
            request: request.clone(), response: response.clone() }];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_returns_a_match_for_multiple_requests() {
    let request = Request { method: s!("GET"), .. Request::default_request() };
    let request2 = Request { method: s!("POST"), path: s!("/post"), .. Request::default_request() };
    let response = Response::default_response();
    let interaction = Interaction { description: s!("test"), provider_states: vec![],
        request: request.clone(), response: response.clone() };
    let interactions = vec![interaction.clone(),
        Interaction { description: s!("test2"), provider_states: vec![],
            request: request2.clone(), response: response.clone() }];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_returns_a_mismatch_for_incorrect_request() {
    let request = Request::default_request();
    let response = Response::default_response();
    let expected_request = Request { query: Some(hashmap!{ s!("QueryA") => vec![s!("Value A")] }),
        .. Request::default_request() };
    let interactions = vec![Interaction { description: s!("test"), provider_states: vec![],
        request: expected_request, response: response.clone() }];
    let result = match_request(&request, &interactions);
    expect!(result.match_key()).to(be_equal_to(s!("Request-Mismatch")));
}

#[test]
fn match_request_returns_request_not_found_if_method_or_path_do_not_match() {
    let request = Request { method: s!("GET"), path: s!("/path"), .. Request::default_request() };
    let response = Response::default_response();
    let expected_request = Request { method: s!("POST"), path: s!("/otherpath"),
        .. Request::default_request() };
    let interactions = vec![Interaction { description: s!("test"), provider_states: vec![],
        request: expected_request, response: response.clone() }];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestNotFound(request)));
}

#[test]
fn match_request_returns_the_most_appropriate_mismatch_for_multiple_requests() {
    let request = Request { method: s!("GET"), path: s!("/"), body: OptionalBody::Present("This is a body".into()),
      .. Request::default_request() };
    let request2 = Request { method: s!("GET"), path: s!("/"), query: Some(hashmap!{
        s!("QueryA") => vec![s!("Value A")]
        }), body: OptionalBody::Present("This is a body".into()),
      .. Request::default_request() };
    let request3 = Request { method: s!("GET"), path: s!("/"), query: Some(hashmap!{
        s!("QueryA") => vec![s!("Value A")]
        }), body: OptionalBody::Missing, .. Request::default_request() };
    let response = Response::default_response();
    let interaction = Interaction { description: s!("test"), provider_states: vec![],
        request: request.clone(), response: response.clone() };
    let interaction2 = Interaction { description: s!("test2"), provider_states: vec![],
            request: request2.clone(), response: response.clone() };
    let interactions = vec![interaction.clone(), interaction2.clone()];
    let result = match_request(&request3, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMismatch(interaction2,
        vec![Mismatch::BodyMismatch { path: s!("/"), expected: Some("This is a body".into()), actual: None,
        mismatch: s!("Expected body \'This is a body\' but was missing") }])));
}

#[test]
fn match_request_supports_v2_matchers() {
    let request = Request { method: s!("GET"), path: s!("/"),
        headers: Some(hashmap!{ s!("Content-Type") => s!("application/json") }), body: OptionalBody::Present(
            r#"
            {
                "a": 100,
                "b": "one hundred"
            }
            "#.into()
        ), .. Request::default_request() };
    let response = Response::default_response();
    let expected_request = Request { method: s!("GET"), path: s!("/"),
        headers: Some(hashmap!{ s!("Content-Type") => s!("application/json") }),
        body: OptionalBody::Present(
            r#"
            {
                "a": 1000,
                "b": "One Thousand"
            }
            "#.into()
        ), matching_rules: matchingrules!{
          "body" => {
            "$.*" => [ MatchingRule::Type ]
          }
        },
      .. Request::default_request()
    };
    let interaction = Interaction { description: s!("test"), provider_states: vec![],
        request: expected_request, response: response.clone() };
    let result = match_request(&request, &vec![interaction.clone()]);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_supports_v2_matchers_with_xml() {
    let request = Request { method: s!("GET"), path: s!("/"), query: None,
        headers: Some(hashmap!{ s!("Content-Type") => s!("application/xml") }), body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>world</foo>
            "#.into()
        ), .. Request::default_request() };
    let response = Response::default_response();
    let expected_request = Request { method: s!("GET"), path: s!("/"), query: None,
        headers: Some(hashmap!{ s!("Content-Type") => s!("application/xml") }),
        body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>mars </foo>
            "#.into()
        ), matching_rules: matchingrules!{
          "body" => {
            "$.foo['#text']" => [ MatchingRule::Regex(s!("[a-z]+")) ]
          }
        },
      .. Request::default_request()
    };
    let interaction = Interaction { description: s!("test"), provider_states: vec![],
        request: expected_request, response: response.clone() };
    let result = match_request(&request, &vec![interaction.clone()]);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}
