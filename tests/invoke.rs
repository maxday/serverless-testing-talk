use serde_json::Value;

#[test]
fn invoke_success() {
    let url = std::env::var("URL").expect("could not read the url");
    let pizza = r#"
        {
            "name": "test-pizza",
            "price": 10
        }"#;
    let pizza: Value = serde_json::from_str(pizza).expect("could not serialize");

    // create
    let client = reqwest::blocking::Client::new();
    let res = client.post(format!("{}/pizza", url))
        .json(&pizza)
        .send()
        .expect("could not send the request");
    assert!(res.status().is_success());

    // get
    let client = reqwest::blocking::Client::new();
    let res = client.get(format!("{}/pizza/test-pizza", url))
        .send()
        .expect("could not send the request");
    assert!(res.status().is_success());
}

#[test]
fn invoke_failure() {
    let url = std::env::var("URL").expect("could not read the url");
    let pizza = r#"
        {
            "invalid": true
        }"#;
    let pizza: Value = serde_json::from_str(pizza).expect("could not serialize");

    // create
    let client = reqwest::blocking::Client::new();
    let res = client.post(format!("{}/pizza", url))
        .json(&pizza)
        .send()
        .expect("could not send the request");
    assert_eq!(res.status(), 400);

    // get
    let client = reqwest::blocking::Client::new();
    let res = client.get(format!("{}/pizza/invalid-pizza", url))
        .send()
        .expect("could not send the request");
    assert_eq!(res.status(), 400);
}