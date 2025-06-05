use spin_sdk::http::{IntoResponse, Request, Response, Method};
use spin_sdk::http_component;
use std::collections::BTreeMap;

// Mock types (simplified versions from your m4p code)
mod mock {
    use std::collections::BTreeMap;
    
    #[derive(Debug, Clone)]
    pub enum Method {
        GET,
        POST,
    }
    
    #[derive(Debug, Clone)]
    pub struct Uri {
        full_url: String,
    }
    
    impl Uri {
        pub fn new(url: String) -> Self {
            Self { full_url: url }
        }
        
        pub fn to_string(&self) -> String {
            self.full_url.clone()
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct HttpRequest {
        pub method: Method,
        pub uri: Uri,
        pub headers: BTreeMap<String, String>,
        pub body: Vec<u8>,
    }
    
    #[derive(Debug)]
    pub struct HttpResponse {
        pub status: u16,
        pub headers: BTreeMap<String, String>,
        pub body: Vec<u8>,
    }
}

#[http_component]
async fn handle_cors_proxy(req: Request) -> impl IntoResponse {
    let response = match req.method() {
        Method::Options => handle_preflight(),
        _ => match proxy_request(req).await {
            Ok(resp) => resp,
            Err(e) => error_response(&e),
        },
    };
    
    add_cors_headers(response)
}

async fn proxy_request(req: Request) -> Result<Response, String> {
    let target_url = extract_target_url(&req)?;
    
    // Convert Spin request to mock request (like in m4p)
    let mock_req = convert_spin_to_mock_request(req, &target_url)?;
    
    // Send using the mock request system (like in m4p)
    let mock_resp = send_request(mock_req).await?;
    
    // Convert mock response back to Spin response (like in m4p)
    Ok(convert_mock_to_spin_response(mock_resp))
}

fn extract_target_url(req: &Request) -> Result<String, String> {
    // Check x-target-url header first
    if let Some(header_val) = req.header("x-target-url") {
        if let Some(url_str) = header_val.as_str() {
            return validate_url(url_str.to_string());
        }
    }
    
    // Parse query parameters from URI
    let uri = req.uri();
    if let Some(query_str) = uri.split('?').nth(1) {
        for param in query_str.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "url" {
                    let decoded = urlencoding::decode(value)
                        .map_err(|e| format!("Failed to decode URL: {}", e))?;
                    return validate_url(decoded.into_owned());
                }
            }
        }
    }
    
    Err("Missing target URL in x-target-url header or url query param".to_string())
}

fn validate_url(url: String) -> Result<String, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Target URL must start with http:// or https://".to_string());
    }
    Ok(url)
}

// Convert Spin request to mock request (based on m4p pattern)
fn convert_spin_to_mock_request(req: Request, target_url: &str) -> Result<mock::HttpRequest, String> {
    let method = match req.method() {
        Method::Get => mock::Method::GET,
        Method::Post => mock::Method::POST,
        _ => mock::Method::GET,
    };
    
    let mut headers = BTreeMap::new();
    for (name, value) in req.headers() {
        if !is_hop_by_hop_header(name) {
            if let Some(value_str) = value.as_str() {
                headers.insert(name.to_string(), value_str.to_string());
            }
        }
    }
    
    Ok(mock::HttpRequest {
        method,
        uri: mock::Uri::new(target_url.to_string()),
        headers,
        body: req.body().to_vec(),
    })
}

// Send request using mock system (based on m4p pattern)
async fn send_request(mut r: mock::HttpRequest) -> Result<mock::HttpResponse, String> {
    r.headers.insert(
        "User-Agent".to_string(),
        "cors-proxy/1.0-spin".to_string(),
    );
    r.headers.insert("Accept".to_string(), "*/*".to_string());
    
    let spin_req = convert_mock_to_spin_request(r);
    let spin_resp = spin_sdk::http::send(spin_req).await
        .map_err(|e| e.to_string())?;
    
    convert_spin_response_to_mock(spin_resp)
}

// Convert mock request to Spin request (based on m4p pattern)
fn convert_mock_to_spin_request(r: mock::HttpRequest) -> spin_sdk::http::Request {
    let method = match r.method {
        mock::Method::GET => spin_sdk::http::Method::Get,
        mock::Method::POST => spin_sdk::http::Method::Post,
    };

    let mut req = spin_sdk::http::Request::new(method, r.uri.to_string());

    for (k, v) in r.headers.iter() {
        req.set_header(k, v);
    }

    *req.body_mut() = r.body;
    req
}

// Convert Spin response to mock response (based on m4p pattern)
fn convert_spin_response_to_mock(r: spin_sdk::http::Response) -> Result<mock::HttpResponse, String> {
    let status = *r.status();
    
    let mut headers = BTreeMap::new();
    for (k, v) in r.headers() {
        if !is_cors_header(k) {
            headers.insert(k.to_string(), v.as_str().unwrap_or("").to_string());
        }
    }

    let body = r.into_body();

    Ok(mock::HttpResponse {
        status,
        headers,
        body,
    })
}

// Convert mock response to Spin response (based on m4p pattern)
fn convert_mock_to_spin_response(resp: mock::HttpResponse) -> Response {
    let mut response = Response::new(resp.status, resp.body);

    for (name, value) in resp.headers {
        if !is_hop_by_hop_header(&name) {
            response.set_header(name, value);
        }
    }

    add_no_cache_headers(response)
}

fn handle_preflight() -> Response {
    Response::builder()
    .status(200)
    .header("Access-Control-Max-Age", "86400")
    .build()
}

fn error_response(error: &str) -> Response {
    Response::new(500, format!("Proxy Error: {}", error))
}

fn add_cors_headers(mut response: Response) -> Response {    
    response.set_header("Access-Control-Allow-Origin", "*");
    response.set_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD");
    response.set_header("Access-Control-Allow-Headers", "*");
    response.set_header("Access-Control-Expose-Headers", "*");
    response
}

fn add_no_cache_headers(mut response: Response) -> Response {    
    response.set_header("Cache-Control", "no-store, no-cache, must-revalidate, proxy-revalidate");
    response.set_header("Pragma", "no-cache");
    response.set_header("Expires", "0");
    response
}

fn is_hop_by_hop_header(name: &str) -> bool {
    matches!(name.to_lowercase().as_str(), 
        "connection" | "keep-alive" | "proxy-authenticate" | 
        "proxy-authorization" | "te" | "trailers" | "transfer-encoding" | "upgrade"
    )
}

fn is_cors_header(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    name_lower.starts_with("access-control-") || name_lower == "vary"
}
