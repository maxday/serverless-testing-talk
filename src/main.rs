
pub mod pizza;

use crate::pizza::{DynamoDBPizzaManager, PizzaManager};
use lambda_runtime::{Error, run, service_fn, LambdaEvent};
use serde_json::{Value, json};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let service = service_fn(handler);
    run(service).await?;
    Ok(())
}

async fn handler(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    let pizza_manager = DynamoDBPizzaManager::new(27017, String::from("pizza")).await;
    let _ = pizza_manager.get(String::from("margherita")).await?;
    Ok(json!({ "statusCode": 200, "body": "Hello world!" }))
}

#[cfg(test)]
mod tests {

    use testcontainers::{clients, self, images};
    use crate::pizza::Pizza;
    use core::time;
    use std::{io::Result, thread};
    use super::*;

    use aws_sdk_dynamodb::model::{
        AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
    };
    
    use async_trait::async_trait;

    #[derive(Default)]
    struct MockedPizzaManager;

    #[async_trait]
    impl PizzaManager for MockedPizzaManager {
        async fn create(&self, _pizza:Pizza) ->  Result<Pizza> {
            Ok(Pizza::new("test-pizza".to_string(), 10))
        }
        async fn get(&self, _pizza_name: String) ->  Result<Option<Pizza>> {
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
        let pizza_manager = DynamoDBPizzaManager::new(port, String::from("pizza_test_2")).await;
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

        let create_table_result = manager.client
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

}
