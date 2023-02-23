pub mod pizza;

use crate::pizza::{DynamoDBPizzaManager, PizzaManager};
use lambda_http::{
    aws_lambda_events::{serde_json::json}, service_fn, Body, Error, IntoResponse, Request,
    RequestExt, Response, http::Method,
};
use pizza::Pizza;
use std::io::Result;

#[tokio::main]
async fn main() -> core::result::Result<(), Error> {
    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn handler(request: Request) -> std::io::Result<impl IntoResponse> {
    let table_name = std::env::var("TABLE_NAME").expect("could not find the table name");
    let pizza_manager = DynamoDBPizzaManager::new(table_name, None).await;
    match request.method() {
        &Method::GET => handle_get(pizza_manager, request).await,
        &Method::POST => handle_post(pizza_manager, request).await,
        _ => Ok(build_error("unsupported"))
    }
}

async fn handle_get(pizza_manager: impl PizzaManager, request: Request) -> Result<Response<Body>> {
    let path_params = request.path_parameters();

    let Some(pizza_name) = path_params.first("pizza_name") else {
        return Ok(build_error("no param found"));
    };
    let pizza = pizza_manager.get(String::from(pizza_name)).await?;
    match pizza {
        None => Ok(build_error("no pizza found")),
        Some(pizza) => Ok(json!(pizza).into_response().await),
    }
}

async fn handle_post(pizza_manager: impl PizzaManager, request: Request) -> Result<Response<Body>> {
    let parse_pizza = request.payload::<Pizza>().unwrap_or_default();
    let Some(pizza) = parse_pizza else {
        return Ok(build_error("could not read the pizza"))
    };
    let pizza = pizza_manager.create(pizza).await?;
    Ok(json!(pizza).into_response().await)
}

fn build_error(error_message: &str) -> Response<Body> {
    Response::builder()
        .status(400)
        .body(lambda_http::Body::from(
            json!({ "error": error_message }).to_string(),
        ))
        .expect("impossible to build the error response")
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::pizza::Pizza;
    use std::io::{Result};
    use lambda_http::http::HeaderValue;
    use testcontainers::{self, clients, images};
    use maplit::hashmap;

    use aws_sdk_dynamodb::{
        model::{
            AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput,
            ScalarAttributeType,
        },
        Client, Config, Credentials, Region,
    };

    use async_trait::async_trait;

    #[derive(Default)]
    struct MockedPizzaManager;

    #[async_trait]
    impl PizzaManager for MockedPizzaManager {
        async fn create(&self, _pizza: Pizza) -> Result<Pizza> {
            Ok(Pizza::new("test-pizza".to_string(), 10))
        }
        async fn get(&self, _pizza_name: String) -> Result<Option<Pizza>> {
            Ok(Some(Pizza::new(String::from("test-pizza"), 10)))
        }
    }

    #[tokio::test]
    async fn test_create_get_pizza_mocked_success() -> Result<()> {
        let pizza_manager = MockedPizzaManager::default();
        let path_params = hashmap! {
            "pizza_name".into() => vec!["deluxe".into()]
        };
        let request = Request::default().with_path_parameters(path_params.clone());
        let result = handle_get(pizza_manager, request).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        let result = std::str::from_utf8(result.body()).unwrap();
        assert_eq!("{\"name\":\"test-pizza\",\"price\":10}", result);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_get_pizza_mocked_error() -> Result<()> {
        let pizza_manager = MockedPizzaManager::default();
        let path_params = hashmap! {
            "invalid_param".into() => vec!["deluxe".into()]
        };
        let request = Request::default().with_query_string_parameters(path_params.clone());
        let result = handle_get(pizza_manager, request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), 400);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_post_pizza_mocked_success() -> Result<()> {
        let pizza_manager = MockedPizzaManager::default();
        let pizza = Pizza::new(String::from("test-pizza"), 10);
        let mut request = Request::new(
            Body::from(serde_json::to_string(&pizza)?));
        request.headers_mut().append("content-type", HeaderValue::from_static("application/json"));
        let result = handle_post(pizza_manager, request).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        let result = std::str::from_utf8(result.body()).unwrap();
        assert_eq!("{\"name\":\"test-pizza\",\"price\":10}", result);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_post_pizza_mocked_error() -> Result<()> {
        let pizza_manager = MockedPizzaManager::default();
        let pizza = "invalid-input";
        let mut request = Request::new(
            Body::from(serde_json::to_string(&pizza)?));
        request.headers_mut().append("content-type", HeaderValue::from_static("application/json"));
        let result = handle_post(pizza_manager, request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), 400);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_get_pizza() -> Result<()> {
        let docker = clients::Cli::default();
        let node = docker.run(images::dynamodb_local::DynamoDb::default());
        let port = node.get_host_port_ipv4(8000);
        let client = build_custom_client(port).await;
        let pizza_manager =
            DynamoDBPizzaManager::new(String::from("pizza_test_2"), Some(client)).await;
        create_db(&pizza_manager).await;
        let pizza = Pizza::new("margherita".to_string(), 10);
        let r = pizza_manager.create(pizza).await?;
        println!("R = {:?}", r);
        let res = pizza_manager.get(String::from("margherita")).await;
        println!("RES = {:?}", res);
        assert!(res.is_ok());
        Ok(())
    }

    async fn create_db(manager: &DynamoDBPizzaManager) {
        let key_schema_hash = KeySchemaElement::builder()
            .attribute_name("name".to_string())
            .key_type(KeyType::Hash)
            .build();

        let attribute_name = AttributeDefinition::builder()
            .attribute_name("name".to_string())
            .attribute_type(ScalarAttributeType::S)
            .build();

        let provisioned_throughput = ProvisionedThroughput::builder()
            .read_capacity_units(5)
            .write_capacity_units(5)
            .build();

        let create_table_result = manager
            .client
            .create_table()
            .table_name(&manager.table_name)
            .key_schema(key_schema_hash)
            .attribute_definitions(attribute_name)
            .provisioned_throughput(provisioned_throughput)
            .send()
            .await;

        assert!(create_table_result.is_ok());
        let req = manager.client.list_tables().limit(1);
        let list_tables_result = req.send().await.unwrap();
        assert_eq!(list_tables_result.table_names().unwrap().len(), 1);
    }

    async fn build_custom_client(port: u16) -> Client {
        let local_credentials = Credentials::new("local", "local", None, None, "local");
        let conf = Config::builder()
            .endpoint_url(format!("http://localhost:{}", port))
            .credentials_provider(local_credentials)
            .region(Region::new("test-region"))
            .build();
        Client::from_conf(conf)
    }
}
