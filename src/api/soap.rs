use serde_json::{json, Value};

fn make_request(body: Value) -> Value {
    json!({
        "Header": {
            "context": {
                "_jsns": "urn:zimbra",
                "userAgent": {
                    "name": "polnio_cyu_email",
                    "version": "0.1.0"
                }
            }
        },
        "Body": body
    })
}

pub fn login(email: &str, password: &str) -> Value {
    make_request(json!({
        "AuthRequest": {
            "_jsns": "urn:zimbraAccount",
            "account": {
                "_content": email,
                "by": "name"
            },
            "password": password
        }
    }))
}
