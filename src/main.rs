use std::io::Result;
pub mod pizza;

use crate::pizza::{DynamoDBPizzaManager, Pizza, PizzaManager};

#[tokio::main]
async fn main() -> Result<()> {
    let pizza_manager = DynamoDBPizzaManager::new("localhost", 27017, "pizza_store", "pizza".to_string()).await;
    let pizza = Pizza::new("margherita".to_string(), 10);
    pizza_manager.create(pizza).await?;
    let result = pizza_manager.get("margherita").await?;
    println!("Pizza: {:?}", result);
    Ok(())
}

#[cfg(test)]
mod tests {

    use testcontainers::{clients, images::mongo::Mongo};
    use super::*;
    use async_trait::async_trait;

    #[derive(Default)]
    struct MockedPizzaManager;

    #[async_trait]
    impl PizzaManager for MockedPizzaManager {
        async fn create(&self, _pizza:Pizza) ->  Result<Pizza> {
            Ok(Pizza::new("test-pizza".to_string(), 1))
        }
        async fn get(&self, _pizza_name:&str) ->  Result<Option<Pizza>> {
            Ok(Some(Pizza::new("test-pizza".to_string(), 1)))
        }
    }

    #[tokio::test]
    async fn test_creat_get_pizza_mocked() -> Result<()> {
        let pizza_manager = MockedPizzaManager::default();
        let pizza = Pizza::new("margherita".to_string(), 10);
        pizza_manager.create(pizza).await?;
        let res = pizza_manager.get("margherita").await;
        assert!(res.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_creat_get_pizza() -> Result<()> {
        let docker = clients::Cli::default();
        let node = docker.run(Mongo);
        let port = node.get_host_port_ipv4(27017);
        let pizza_manager = DynamoDBPizzaManager::new("localhost", port, "pizza_store_test", "pizza_test".to_string()).await;
        let pizza = Pizza::new("margherita".to_string(), 10);
        pizza_manager.create(pizza).await?;
        let res = pizza_manager.get("margherita").await;
        assert!(res.is_ok());
        Ok(())
    }

}
