#!/usr/bin/env cargo
use wire_api::openapi::WireV1ApiDoc;

fn main() {
    let openapi = WireV1ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi)
        .expect("Failed to serialize OpenAPI spec to JSON");

    println!("{}", json);
}
