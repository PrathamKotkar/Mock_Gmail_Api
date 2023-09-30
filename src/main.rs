use reqwest;
use serde_json::Value;
use tiny_http::{Server, Response};
use url::Url;
use open;
use serde::{Deserialize, Serialize};

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Messages{
    id : String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Mail{
    messages: Vec<Messages>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code_request_params = [
        ("scope", "https://mail.google.com"),
        ("access_type", "offline"),
        ("include_granted_scopes", "true"),
        ("response_type", "code"),
        ("redirect_uri", "http://localhost:8080"),
        ("state", "state_parameter_passthrough_value"),
        (
            "client_id",
            "319107843868-cnb8ko19g0rqo1a3569juftrrkt42ed6.apps.googleusercontent.com",
        ),
    ];

    let auth_url = reqwest::blocking::Client::new()
        .get(AUTH_URL)
        .query(&code_request_params)
        .build()?
        .url()
        .to_string();

    open::that(auth_url)?;

    let server = Server::http("localhost:8080").expect("Failed to create server");

    let mut code = String::new();

    for request in server.incoming_requests() {
        let response = Response::from_string("close this window and return to the terminal.");

        let redirected_url = request.url();
        code = extract_code_from_url("http://localhost:8080", redirected_url)?;

        request.respond(response).expect("Failed to respond to request");

        break;
    }

    let client_id = "319107843868-cnb8ko19g0rqo1a3569juftrrkt42ed6.apps.googleusercontent.com".to_string();
    let client_secret = "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".to_string();
    let grant_type = "authorization_code".to_string();
    let redirect_uri = "http://localhost:8080".to_string();
    let token_request_params = [
        ("code", &code),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("redirect_uri", &redirect_uri),
        ("grant_type", &grant_type),
    ];

    let token_response = reqwest::blocking::Client::new()
        .post(TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&token_request_params)
        .send()?;
    
    if token_response.status().is_success() {
        let token_data: Value = token_response.json()?;
        if let Some(access_token) = token_data.get("access_token") {
            if let Some(access_token_str) = access_token.as_str() {
                
                let gmail_api_url = "https://gmail.googleapis.com/gmail/v1/users/me/messages";
                let api_response = reqwest::blocking::Client::new()
                    .get(gmail_api_url)
                    .header("Authorization", format!("Bearer {}", access_token_str))
                    .send()?;
                
                    if api_response.status().is_success() {
                        let response_text = api_response.text()?;
                        let messages: Mail = serde_json::from_str(&response_text)?;
                
                        println!("Gmail API Response (Tabular format):\n");
                
                        println!("{:<20}", "Message ID");
                        println!("{:-<16}", "");
                
                        for message in &messages.messages {
                            println!("{:<20}", message.id);
                        }
                    } else {
                        eprintln!("Failed to fetch Gmail API data. Status code: {}", api_response.status());
                    }
            } else {
                eprintln!("Access token is not a string.");
            }
        } else {
            eprintln!("Access token not found in the response.");
        }
    } else {
        eprintln!("Failed to obtain access token. Status code: {}", token_response.status());
    }

    Ok(())
}

fn extract_code_from_url(base_url: &str, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let base_url = Url::parse(base_url)?;
    let full_url = base_url.join(url)?;
    let code = full_url.query_pairs().find(|(key, _)| key == "code");

    match code {
        Some((_, value)) => Ok(value.to_string()),
        None => Err("Code not found in the redirected URL.".into()),
    }
}
