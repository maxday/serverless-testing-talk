use mongodb::bson::{doc, Document};
use mongodb::Database;
use mongodb::{options::ClientOptions, Client};
use async_trait::async_trait;
use std::io::Result;

#[derive(Debug)]
pub struct Pizza {
    name: String,
    price: i32,
}

impl Pizza {
    pub fn new(name: String, price: i32) -> Self {
        Pizza { name, price }
    }
    fn to_document(&self) -> Document {
        doc! { "name": &self.name, "price": self.price }
    }
    fn from_document(document: Option<Document>) -> Option<Self> {
        match document {
            None => None,
            Some(document) => {
                let name = document.get_str("name".to_string()).expect("could not find the name");
                let price = document.get_i32("price".to_string()).expect("could not find the price");
                Some(Pizza {
                    name: name.to_string(),
                    price,
                })
            }
        }
    }
}

#[async_trait]
pub trait PizzaManager {
    async fn create(&self, pizza: Pizza) -> Result<Pizza>;
    async fn get(&self, pizza_name: &str) -> Result<Option<Pizza>>;
}

pub struct DynamoDBPizzaManager {
    db: Database,
    collection_name: String
}
impl DynamoDBPizzaManager {
    pub async fn new(host: &str, port: u16, db_name: &str, collection_name: String) -> Self {
        let client_options = ClientOptions::parse(format!("mongodb://{}:{}", host, port)).await.expect("could not parse the args ");
        let client = Client::with_options(client_options).expect("could not build the client");
        let db = client.database(db_name);
        DynamoDBPizzaManager {
            db,
            collection_name
        }
    }
}

#[async_trait]
impl PizzaManager for DynamoDBPizzaManager {
    async fn create(&self, pizza: Pizza) -> Result<Pizza> {
        let collection = self.db.collection::<Document>(&self.collection_name);
        match collection.insert_one(pizza.to_document(), None).await {
            Ok(_) => Ok(pizza),
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "could not create the pizza")),
        }
    }

    async fn get(&self, pizza_name: &str) -> Result<Option<Pizza>> {
        let collections = self.db.collection::<Document>(&self.collection_name);
        let filter = doc! { "name": pizza_name };
        match collections.find_one(Some(filter), None).await {
            Ok(document) => Ok(Pizza::from_document(document)),
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "could not find the pizza")),
        }
    }
}
