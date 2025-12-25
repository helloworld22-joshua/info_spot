use std::net::TcpListener;
use std::io::{Read, Write};
use std::sync::mpsc;

pub fn start_callback_server() -> Result<String, Box<dyn std::error::Error>> {
    // Get port from REDIRECT_URI env variable, default to 8888
    let redirect_uri = std::env::var("SPOTIFY_REDIRECT_URI")
        .unwrap_or_else(|_| "http://127.0.0.1:8888/callback".to_string());
    
    // Extract port from URI (e.g., "http://127.0.0.1:8080/callback" -> "8080")
    let port = redirect_uri
        .split(':')
        .nth(2)
        .and_then(|s| s.split('/').next())
        .unwrap_or("8888");
    
    let bind_addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&bind_addr)?;
    println!("Callback server listening on http://{}", bind_addr);

    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0; 1024];
            if let Ok(size) = stream.read(&mut buffer) {
                let request = String::from_utf8_lossy(&buffer[..size]);
                
                if let Some(code) = extract_code_from_request(&request) {
                    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                        <html><body><h1>Authentication successful!</h1>\
                        <p>You can close this window and return to the app.</p>\
                        <script>window.close();</script></body></html>";
                    let _ = stream.write_all(response.as_bytes());
                    let _ = tx.send(code);
                }
            }
        }
    });

    // Wait for the code with a timeout
    match rx.recv_timeout(std::time::Duration::from_secs(300)) {
        Ok(code) => Ok(code),
        Err(_) => Err("Timeout waiting for OAuth callback".into()),
    }
}

fn extract_code_from_request(request: &str) -> Option<String> {
    // Parse the request line to get the URL
    let first_line = request.lines().next()?;
    let url_part = first_line.split_whitespace().nth(1)?;
    
    // Extract the code parameter
    if let Some(query_start) = url_part.find('?') {
        let query = &url_part[query_start + 1..];
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "code" {
                    return Some(value.to_string());
                }
            }
        }
    }
    
    None
}