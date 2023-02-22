pub mod pizza;

use crate::pizza::{DynamoDBPizzaManager, PizzaManager};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use pizza::Pizza;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let service = service_fn(handler);
    run(service).await?;
    Ok(())
}

async fn handler(_event: LambdaEvent<Pizza>) -> Result<Pizza, Error> {
    let table_name = std::env::var("PIZZA_NAME").expect("could not find the table name");
    let pizza_manager = DynamoDBPizzaManager::new(table_name, None).await;
    let pizza = pizza_manager.get(String::from("margherita")).await?;
    match pizza {
        None => panic!("could not find the pizza"),
        Some(pizza) => Ok(pizza),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::pizza::Pizza;
    use std::io::Result;
    use testcontainers::{self, clients, images};

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
            //Ok(None)
        }
    }

    #[tokio::test]
    async fn test_create_get_pizza_mocked() -> Result<()> {
        let pizza_manager = MockedPizzaManager::default();
        let pizza = Pizza::new(String::from("margherita"), 10);
        pizza_manager.create(pizza).await?;
        let res = pizza_manager.get(String::from("margherita")).await?;
        assert!(res.is_some());
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
            .region(Region::new("us-east-1"))
            .build();
        Client::from_conf(conf)
    }
}
