use anyhow::{Context, Result};
use std::time::Duration;
use tracing::{info, warn};
use tokio::time::sleep;
use std::env;

/// OAuth flow handler for providers that use browser-based authentication
pub struct OAuthHandler {
    provider: String,
    client_id: String,
    redirect_uri: String,
    auth_endpoint: String,
}

impl OAuthHandler {
    /// Create a new OAuth handler for Anthropic Pro/Max
    pub fn new_anthropic_pro() -> Self {
        Self {
            provider: "anthropic-pro".to_string(),
            client_id: "aircher-cli".to_string(),
            redirect_uri: "http://localhost:8765/callback".to_string(),
            auth_endpoint: "https://claude.ai/oauth/authorize".to_string(),
        }
    }

    /// Start the OAuth flow
    pub async fn start_auth_flow(&self) -> Result<String> {
        // Generate a random state parameter for security
        let state = Self::generate_state();
        
        // Build the authorization URL
        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&state={}",
            self.auth_endpoint,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&state)
        );

        info!("🌐 Starting OAuth flow for {}", self.provider);
        
        // Try to open the browser
        match Self::open_browser(&auth_url) {
            Ok(()) => {
                info!("✓ Opened browser for authentication");
                Ok(auth_url)
            }
            Err(e) => {
                warn!("Failed to open browser: {}", e);
                // Return the URL for manual opening
                Ok(auth_url)
            }
        }
    }

    /// Check if we're in an SSH session
    pub fn is_ssh_session() -> bool {
        // Check common SSH environment variables
        env::var("SSH_CLIENT").is_ok() || 
        env::var("SSH_TTY").is_ok() || 
        env::var("SSH_CONNECTION").is_ok()
    }

    /// Open a URL in the default browser
    fn open_browser(url: &str) -> Result<()> {
        // Platform-specific browser opening
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(url)
                .spawn()
                .context("Failed to open browser on macOS")?;
        }

        #[cfg(target_os = "linux")]
        {
            // Try xdg-open first, then fallback to other options
            if std::process::Command::new("xdg-open")
                .arg(url)
                .spawn()
                .is_err()
            {
                // Try other common Linux browsers
                for browser in &["firefox", "chromium", "google-chrome", "sensible-browser"] {
                    if std::process::Command::new(browser)
                        .arg(url)
                        .spawn()
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                return Err(anyhow::anyhow!("No suitable browser found"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "start", "", url])
                .spawn()
                .context("Failed to open browser on Windows")?;
        }

        Ok(())
    }

    /// Generate a random state parameter for OAuth security
    fn generate_state() -> String {
        use rand::{distributions::Alphanumeric, Rng};
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    /// Start a local HTTP server to receive the OAuth callback
    pub async fn start_callback_server(&self, state: &str) -> Result<String> {
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:8765")
            .await
            .context("Failed to start OAuth callback server")?;

        info!("📡 Waiting for OAuth callback on http://localhost:8765");

        // Set a timeout for the OAuth flow
        let timeout_duration = Duration::from_secs(300); // 5 minutes
        
        tokio::select! {
            result = self.wait_for_callback(listener, state) => {
                result
            }
            _ = sleep(timeout_duration) => {
                Err(anyhow::anyhow!("OAuth flow timed out after 5 minutes"))
            }
        }
    }

    /// Wait for the OAuth callback
    async fn wait_for_callback(&self, listener: tokio::net::TcpListener, expected_state: &str) -> Result<String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        loop {
            let (mut socket, _) = listener.accept().await?;
            
            let mut buffer = [0; 1024];
            let n = socket.read(&mut buffer).await?;
            let request = String::from_utf8_lossy(&buffer[..n]);
            
            // Parse the request to extract the code and state
            if let Some(code) = Self::extract_param(&request, "code") {
                if let Some(state) = Self::extract_param(&request, "state") {
                    if state == expected_state {
                        // Send success response
                        let response = "HTTP/1.1 200 OK\r\n\
                                      Content-Type: text/html\r\n\
                                      \r\n\
                                      <html><body>\
                                      <h1>Authentication Successful!</h1>\
                                      <p>You can now close this window and return to Aircher.</p>\
                                      <script>window.close();</script>\
                                      </body></html>";
                        
                        socket.write_all(response.as_bytes()).await?;
                        socket.flush().await?;
                        
                        info!("✓ Received OAuth callback with authorization code");
                        return Ok(code);
                    } else {
                        warn!("OAuth callback received with invalid state");
                    }
                }
            }
            
            // Send error response
            let response = "HTTP/1.1 400 Bad Request\r\n\
                          Content-Type: text/html\r\n\
                          \r\n\
                          <html><body>\
                          <h1>Authentication Failed</h1>\
                          <p>Invalid or missing authorization code.</p>\
                          </body></html>";
            
            socket.write_all(response.as_bytes()).await?;
            socket.flush().await?;
        }
    }

    /// Extract a parameter from a URL query string
    fn extract_param(request: &str, param_name: &str) -> Option<String> {
        // Find the GET line
        let get_line = request.lines().find(|line| line.starts_with("GET"))?;
        
        // Extract the path
        let path = get_line.split_whitespace().nth(1)?;
        
        // Find the query string
        let query_start = path.find('?')?;
        let query = &path[query_start + 1..];
        
        // Parse query parameters
        for pair in query.split('&') {
            let mut parts = pair.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if key == param_name {
                    return Some(urlencoding::decode(value).ok()?.into_owned());
                }
            }
        }
        
        None
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(&self, code: &str) -> Result<String> {
        let client = reqwest::Client::new();
        
        let token_endpoint = "https://claude.ai/oauth/token";
        
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", &self.client_id),
            ("redirect_uri", &self.redirect_uri),
        ];
        
        let response = client
            .post(token_endpoint)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange authorization code")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Token exchange failed: {}", error_text));
        }
        
        let token_response: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse token response")?;
        
        token_response["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No access token in response"))
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_generation() {
        let state1 = OAuthHandler::generate_state();
        let state2 = OAuthHandler::generate_state();
        
        assert_eq!(state1.len(), 32);
        assert_eq!(state2.len(), 32);
        assert_ne!(state1, state2);
    }

    #[test]
    fn test_param_extraction() {
        let request = "GET /callback?code=test123&state=abc123 HTTP/1.1\r\n\
                      Host: localhost:8765\r\n\
                      \r\n";
        
        assert_eq!(OAuthHandler::extract_param(request, "code"), Some("test123".to_string()));
        assert_eq!(OAuthHandler::extract_param(request, "state"), Some("abc123".to_string()));
        assert_eq!(OAuthHandler::extract_param(request, "missing"), None);
    }
}