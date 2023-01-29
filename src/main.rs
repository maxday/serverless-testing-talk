use mongodb::bson::{doc, Document};
use mongodb::error::Error;
use mongodb::results::InsertOneResult;
use mongodb::Database;
use mongodb::{options::ClientOptions, Client};

async fn create_pizza(
    db: &Database,
    collection_name: &str,
    pizza: Document,
) -> Result<InsertOneResult, Error> {
    let collection = db.collection::<Document>(collection_name);
    collection.insert_one(pizza, None).await
}

async fn get_pizza_by_name(
    db: &Database,
    collection_name: &str,
    pizza_name: &str,
) -> Result<Option<Document>, Error> {
    let collections = db.collection::<Document>(collection_name);
    let filter = doc! { "name": pizza_name };
    collections.find_one(Some(filter), None).await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let database_name = "pizza_store";
    let collection_name = "pizza";

    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = client.database(database_name);

    let pizza =
        doc! { "name": "margherita", "topings": ["tomato sauce", "fior di latte", "basel"] };

    create_pizza(&db, &collection_name, pizza).await?;
    let result = get_pizza_by_name(&db, &collection_name, "margherita").await?;

    println!("Pizza: {}", result.unwrap_or_default());
    Ok(())
}

// #[cfg(test)]
// mod tests {

//     use testcontainers::clients;
//     use super::*;

//     #[tokio::test]
//     async fn test_insert_pizza() -> Result<(), Error> {
//         let docker = clients::Cli::default();
//         let node = docker.run(auth_mock::AuthMock::default());
//         let port = node.get_host_port_ipv4(3000);

//         let fake_repo = TestDbData {};
//         let auth_config = Data::new(AuthConfig::new(format!("http://0.0.0.0:{}/user", port)));
//         let app = test::init_service(
//             app!(Some("emptyDb"), Data::clone(&auth_config), TestDbData)
//                 .app_data(web::Data::new(fake_repo).clone()),
//         )
//         .await;
//         let course = Course {
//             uuid: String::from("test-uuid"),
//             user_id: String::from("100"),
//             name: String::from("test-name"),
//         };
//         let req = test::TestRequest::post()
//             .set_json(&course)
//             .uri("/courses")
//             .append_header((AUTHORIZATION, "Bearer valid-test-token"))
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert!(resp.status().is_success());

//         let req = test::TestRequest::get()
//             .uri("/courses/test-uuid")
//             .append_header((AUTHORIZATION, "Bearer valid-test-token"))
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert!(resp.status().is_success());
//         Ok(())
//     }

// }

// client_options.app_name = Some("My App".to_string());
// client_options.connect_timeout = Some(Duration::from_secs(1));
// client_options.server_selection_timeout = Some(Duration::from_secs(1));
