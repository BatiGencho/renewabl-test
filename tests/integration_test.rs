use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};

fn build_test_server() -> TestServer {
    let store = renewabl_api::store::PlantStore::new();
    let app = renewabl_api::routes::app(store);
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_list_plants_empty() {
    let server = build_test_server();
    let response = server.get("/plants").await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body, json!([]));
}

#[tokio::test]
async fn test_create_plant() {
    let server = build_test_server();
    let response = server
        .post("/plants")
        .json(&json!({
            "name": "Sunny Farm",
            "energy_type": "solar",
            "capacity_mw": 50.0,
            "location": "California, USA"
        }))
        .await;
    response.assert_status(StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "Sunny Farm");
    assert_eq!(body["energy_type"], "solar");
    assert_eq!(body["capacity_mw"], 50.0);
    assert_eq!(body["location"], "California, USA");
    assert_eq!(body["status"], "active");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_get_plant() {
    let server = build_test_server();

    let create_response = server
        .post("/plants")
        .json(&json!({
            "name": "Wind Valley",
            "energy_type": "wind",
            "capacity_mw": 120.0,
            "location": "Texas, USA"
        }))
        .await;
    let created: Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    let get_response = server.get(&format!("/plants/{id}")).await;
    get_response.assert_status_ok();
    let body: Value = get_response.json();
    assert_eq!(body["id"], id);
    assert_eq!(body["name"], "Wind Valley");
}

#[tokio::test]
async fn test_get_plant_not_found() {
    let server = build_test_server();
    let fake_id = "00000000-0000-0000-0000-000000000000";
    let response = server.get(&format!("/plants/{fake_id}")).await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_plant() {
    let server = build_test_server();

    let create_response = server
        .post("/plants")
        .json(&json!({
            "name": "Hydro Peak",
            "energy_type": "hydro",
            "capacity_mw": 200.0,
            "location": "Norway"
        }))
        .await;
    let created: Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    let update_response = server
        .put(&format!("/plants/{id}"))
        .json(&json!({
            "status": "maintenance",
            "capacity_mw": 180.0
        }))
        .await;
    update_response.assert_status_ok();
    let updated: Value = update_response.json();
    assert_eq!(updated["status"], "maintenance");
    assert_eq!(updated["capacity_mw"], 180.0);
    assert_eq!(updated["name"], "Hydro Peak");
}

#[tokio::test]
async fn test_delete_plant() {
    let server = build_test_server();

    let create_response = server
        .post("/plants")
        .json(&json!({
            "name": "Geo Station",
            "energy_type": "geothermal",
            "capacity_mw": 30.0,
            "location": "Iceland"
        }))
        .await;
    let created: Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    let delete_response = server.delete(&format!("/plants/{id}")).await;
    delete_response.assert_status(StatusCode::NO_CONTENT);

    let get_response = server.get(&format!("/plants/{id}")).await;
    get_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_plants_after_create() {
    let server = build_test_server();

    server
        .post("/plants")
        .json(&json!({
            "name": "Tidal Force",
            "energy_type": "tidal",
            "capacity_mw": 10.0,
            "location": "Scotland, UK"
        }))
        .await;

    server
        .post("/plants")
        .json(&json!({
            "name": "Bio Energy",
            "energy_type": "biomass",
            "capacity_mw": 25.0,
            "location": "Sweden"
        }))
        .await;

    let response = server.get("/plants").await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body.as_array().unwrap().len(), 2);
}
