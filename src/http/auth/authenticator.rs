use std::env;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response, routing::get, Router};
use openidconnect::{core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata, CoreUserInfoClaims}, AccessToken, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

use crate::{config::Config, http::auth::user::CurrentUser};


#[derive(Clone)]
pub struct Authenticator {
    http_client: Client,
    client:  openidconnect::Client<openidconnect::EmptyAdditionalClaims, openidconnect::core::CoreAuthDisplay, openidconnect::core::CoreGenderClaim, openidconnect::core::CoreJweContentEncryptionAlgorithm, openidconnect::core::CoreJsonWebKey, openidconnect::core::CoreAuthPrompt, openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>, openidconnect::StandardTokenResponse<openidconnect::IdTokenFields<openidconnect::EmptyAdditionalClaims, openidconnect::EmptyExtraTokenFields, openidconnect::core::CoreGenderClaim, openidconnect::core::CoreJweContentEncryptionAlgorithm, openidconnect::core::CoreJwsSigningAlgorithm>, openidconnect::core::CoreTokenType>, openidconnect::StandardTokenIntrospectionResponse<openidconnect::EmptyExtraTokenFields, openidconnect::core::CoreTokenType>, openidconnect::core::CoreRevocableToken, openidconnect::StandardErrorResponse<openidconnect::RevocationErrorResponseType>, openidconnect::EndpointSet, openidconnect::EndpointNotSet, openidconnect::EndpointNotSet, openidconnect::EndpointNotSet, openidconnect::EndpointMaybeSet, openidconnect::EndpointMaybeSet>,
    self_url: String
}

impl Authenticator {

    pub async fn create(config: &Config) -> Self {
        let http_client = Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(config.oidc_url.clone()).unwrap(),
            &http_client
        ).await.unwrap();

        let client=
            CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.oidc_client_secret.clone()),
            Some(ClientSecret::new(config.oidc_client_id.clone())),
        )
        .set_redirect_uri(RedirectUrl::new(env::var("OIDC_REDIRECT_URL").unwrap_or("http://localhost:5000/".to_string())).unwrap());

        return Authenticator {
            http_client:  http_client,
            client: client,
            self_url: config.self_url.clone(),
        }
    }
    
    pub fn get_redirect_url(&self, state: String) -> (PkceCodeVerifier, (reqwest::Url, CsrfToken, Nonce)) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        return (pkce_verifier, self.client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                || {CsrfToken::new(state)},
                Nonce::new_random,
            )
            // Set the desired scopes.
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            // Set the PKCE code challenge.
            //.set_pkce_challenge(pkce_challenge)
            .url());
    }

    pub async fn get_from_redirected(&self, token: String, csrf: String) -> AccessToken {

        // let pkce_verifier = PkceCodeVerifier::new(csrf);

        let token_response =self.client
        .exchange_code(AuthorizationCode::new(token)).unwrap()
        // Set the PKCE code verifier.
        // .set_pkce_verifier(pkce_verifier)
        .request_async(&self.http_client).await.unwrap();

        return token_response.access_token().clone();
    }

    async fn authorize(&self, str: &str) -> Option<CurrentUser> {
        if let Ok(result) = self.client.user_info(AccessToken::new(str.to_string()), None) {
            let resulting: Result<CoreUserInfoClaims, _> = result.request_async(&self.http_client).await;
            
            if let Ok(requested) = resulting {
                let id = requested.subject().clone().to_string();

                return Some(CurrentUser {
                    id: id
                });
            }
        }

        return None;
    }

    pub async fn middleware(&self, mut req: Request, next: Next) -> Result<Response, StatusCode> {
        let mut auth_header = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|header| header.to_str().ok())
                .ok_or(StatusCode::UNAUTHORIZED)?
                .to_string();

            if auth_header.starts_with("Bearer")  {
                auth_header = auth_header.strip_prefix("Bearer").unwrap_or(&auth_header).to_string();
            }

            auth_header = auth_header.trim().to_string();

            if let Some(user) = self.authorize(&auth_header).await {
                req.extensions_mut().insert(user);
                return Ok(next.run(req).await);
            }

        return Err(StatusCode::UNAUTHORIZED);
    }
}