use grio::*;
use std::time::Duration;

#[tokio::test]
async fn test_api_schema_and_predict_flow() {
    let app = App::new("Test App")
        .item(Text::new("a").label("A").value("Hello"))
        .item(Slider::new("b").label("B").min(1.0).max(10.0).value(3.0))
        .item(Output::new("c").label("C"))
        .api_key("secret123")
        .on_submit(|ctx| {
            let a: String = ctx.get("a")?;
            let b: f64 = ctx.get("b")?;
            ctx.set("c", format!("{a} x{b}"));
            Ok(())
        });

    let port = 17865;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    // Laisser le serveur démarrer
    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. Test GET /api/schema
    let schema = http_get(&format!("http://127.0.0.1:{port}/api/schema")).await;
    assert!(schema.contains("predict"));
    assert!(schema.contains("openapi"));

    // 2. Test GET /api/openapi.json
    let openapi = http_get(&format!("http://127.0.0.1:{port}/api/openapi.json")).await;
    assert!(openapi.contains("openapi"));
    assert!(openapi.contains("3.0.3"));

    // 3. Test GET /docs
    let docs = http_get(&format!("http://127.0.0.1:{port}/docs")).await;
    assert!(docs.contains("swagger-ui"));

    // 4. Test POST /api/predict sans clé (doit échouer 401)
    let unauth = http_post(&format!("http://127.0.0.1:{port}/api/predict"), r#"{"data":["Ada",4]}"#, None).await;
    assert!(unauth.contains("401 Unauthorized") || unauth.contains("Invalid or missing API key"));

    // 5. Test POST /api/predict avec clé (doit réussir 200)
    let auth_ok = http_post(&format!("http://127.0.0.1:{port}/api/predict"), r#"{"data":["Ada",4]}"#, Some("secret123")).await;
    assert!(auth_ok.contains("Ada x4"));
}

async fn http_get(url: &str) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let url = url.strip_prefix("http://").unwrap();
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    let path = format!("/{path}");

    let mut stream = TcpStream::connect(host).await.unwrap();
    let req = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await.unwrap();

    let mut resp = String::new();
    stream.read_to_string(&mut resp).await.unwrap();
    resp
}

async fn http_post(url: &str, body: &str, api_key: Option<&str>) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let url = url.strip_prefix("http://").unwrap();
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    let path = format!("/{path}");

    let mut stream = TcpStream::connect(host).await.unwrap();
    let auth_header = if let Some(key) = api_key {
        format!("X-API-Key: {key}\r\n")
    } else {
        String::new()
    };

    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth_header}Connection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(req.as_bytes()).await.unwrap();

    let mut resp = String::new();
    stream.read_to_string(&mut resp).await.unwrap();
    resp
}
