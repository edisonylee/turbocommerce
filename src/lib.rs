use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;


#[http_component]
fn handle_hello_wasm(_req: Request) -> anyhow::Result<Response> {
let html = r#"
<html>
<head><title>Hello WASM</title></head>
<body>
<h1>Hello from WASM</h1>
<p>This is running on Spin.</p>
</body>
</html>
"#;


Ok(
Response::builder()
.status(200)
.header("content-type", "text/html")
.body(html)
.build()
)
}