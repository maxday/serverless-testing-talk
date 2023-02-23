use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct Pizza {
    pub name: String,
    price: i32,
}

#[async_trait]
pub trait PizzaManager {
    async fn create(&self, pizza: Pizza) -> Result<Pizza>;
    async fn get(&self, pizza_name: String) -> Result<Option<Pizza>>;
}

pub struct DynamoDBPizzaManager {
    pub client: Client,
    pub table_name: String,
}
impl DynamoDBPizzaManager {
    pub async fn new(table_name: String, client: Option<Client>) -> Self {
        let client = match client {
            Some(client) => client,
            None => {
                let shared_config = aws_config::load_from_env().await;
                aws_sdk_dynamodb::Client::new(&shared_config)
            }
        };
        DynamoDBPizzaManager { client, table_name }
    }
}

#[async_trait]
impl PizzaManager for DynamoDBPizzaManager {
    async fn create(&self, pizza: Pizza) -> Result<Pizza> {
        let name = AttributeValue::S(pizza.name.to_string());
        let price = AttributeValue::N(pizza.price.to_string());

        let command = self
            .client
            .put_item()
            .table_name(&self.table_name)
            .item("name", name)
            .item("price", price)
            .send();

        match command.await {
            Ok(_) => Ok(pizza),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                e,
            )),
        }
    }

    async fn get(&self, pizza_name: String) -> Result<Option<Pizza>> {
        let command = self
            .client
            .query()
            .table_name(&self.table_name)
            .expression_attribute_names("#pizza_name", "name")
            .expression_attribute_values(":name", AttributeValue::S(pizza_name))
            .key_condition_expression("#pizza_name = :name")
            .send()
            .await;

        let Ok(results) = command else {
            return Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "could not get the pizza"));
        };
        let Some(items) = results.items else {
            return Ok(None);
        };
        let Some(pizza) = items.first() else {
            return Ok(None);
        };
        Ok(Some(Pizza::from(pizza)))
    }
}

impl Pizza {
    pub fn new(name: String, price: i32) -> Self {
        Pizza { name, price }
    }
    fn from(value: &HashMap<String, AttributeValue>) -> Pizza {
        Pizza {
            name: value
                .get("name")
                .expect("could not find name")
                .as_s()
                .expect("wrong type for name")
                .to_string(),
            price: value
                .get("price")
                .expect("could not find price")
                .as_n()
                .expect("wrong type for price")
                .to_string()
                .parse::<i32>()
                .expect("could not get the price"),
        }
    }
}
