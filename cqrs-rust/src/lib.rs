#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]

use anyhow::bail;
use cqrs_commands::{Commands, CreateProductModel, UpdateProductModel};
use cqrs_queries::Queries;
use spin_sdk::http::{IntoResponse, Params, Request, Response, Router};
use spin_sdk::http_component;

#[http_component]
fn handle_cqrs(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::default();

    // register routes for queries
    router.get("/items", query_all_products);
    router.get("/items/:id", query_product_by_id);

    // register routes for commands
    router.post("/items", create_product);
    router.put("/items/:id", update_product_by_id);
    router.delete("/items/:id", delete_product_by_id);

    // handle all the requests
    Ok(router.handle(req))
}

fn query_all_products(_: Request, _: Params) -> anyhow::Result<impl IntoResponse> {
    match Queries::all_products() {
        Ok(p) => {
            let payload = serde_json::to_vec(&p)?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(payload)
                .build())
        }
        Err(e) => bail!(e),
    }
}

fn query_product_by_id(_: Request, params: Params) -> anyhow::Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(Response::builder().status(400).body(()).build());
    };
    match Queries::product_by_id(id.to_string()) {
        Ok(p) => {
            let payload = serde_json::to_vec(&p)?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(payload)
                .build())
        }
        Err(e) => bail!(e),
    }
}

fn create_product(req: Request, _: Params) -> anyhow::Result<impl IntoResponse> {
    let Ok(model) = serde_json::from_slice::<CreateProductModel>(req.body()) else {
        return Ok(Response::builder().status(400).body(()).build());
    };

    let product = Commands::create_product(model)?;
    let payload = serde_json::to_vec(&product)?;
    return Ok(Response::builder()
        .status(201)
        .header("Content-Type", "application/json")
        .header(
            "Location",
            build_location_header_value(req.uri(), product.id),
        )
        .body(payload)
        .build());
}

fn update_product_by_id(req: Request, params: Params) -> anyhow::Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(Response::builder().status(400).body(()).build());
    };
    let Ok(model) = serde_json::from_slice::<UpdateProductModel>(req.body()) else {
        return Ok(Response::builder().status(400).body(()).build());
    };

    match Commands::update_product(id.to_string(), model) {
        Ok(p) => {
            let payload = serde_json::to_vec(&p)?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(payload)
                .build())
        }
        Err(e) => bail!(e),
    }
}

fn delete_product_by_id(_: Request, params: Params) -> anyhow::Result<impl IntoResponse> {
    let Some(id) = params.get("id") else {
        return Ok(Response::builder().status(400).body(()).build());
    };
    match Commands::delete_product_by_id(id.to_string()) {
        Ok(true) => Ok(Response::builder().status(204).body(()).build()),
        Ok(false) => Ok(Response::builder().status(404).body(()).build()),
        Err(e) => bail!(e),
    }
}

fn build_location_header_value(uri: &str, id: String) -> String {
    if uri.ends_with("/") {
        return format!("{}{}", uri, id);
    }
    format!("{}/{}", uri, id)
}
