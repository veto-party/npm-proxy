use std::env;

use axum::{extract::Request, middleware::Next, response::Response, http::StatusCode};
use openidconnect::{core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata, CoreUserInfoClaims}, AccessToken, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope};
use reqwest::{header, Client};

use crate::http::auth::user::CurrentUser;


#[derive(Clone)]
pub struct Authenticator {
    http_client: Client,
    client:  openidconnect::Client<openidconnect::EmptyAdditionalClaims, openidconnect::core::CoreAuthDisplay, openidconnect::core::CoreGenderClaim, openidconnect::core::CoreJweContentEncryptionAlgorithm, openidconnect::core::CoreJsonWebKey, openidconnect::core::CoreAuthPrompt, openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>, openidconnect::StandardTokenResponse<openidconnect::IdTokenFields<openidconnect::EmptyAdditionalClaims, openidconnect::EmptyExtraTokenFields, openidconnect::core::CoreGenderClaim, openidconnect::core::CoreJweContentEncryptionAlgorithm, openidconnect::core::CoreJwsSigningAlgorithm>, openidconnect::core::CoreTokenType>, openidconnect::StandardTokenIntrospectionResponse<openidconnect::EmptyExtraTokenFields, openidconnect::core::CoreTokenType>, openidconnect::core::CoreRevocableToken, openidconnect::StandardErrorResponse<openidconnect::RevocationErrorResponseType>, openidconnect::EndpointSet, openidconnect::EndpointNotSet, openidconnect::EndpointNotSet, openidconnect::EndpointNotSet, openidconnect::EndpointMaybeSet, openidconnect::EndpointMaybeSet>,
}

impl Authenticator {

    pub async fn create() -> Authenticator {
        let http_client = Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new( env::var("OIDC_ISSUER_URL").unwrap_or("https://gitlab.git.veto.dev".to_string())).unwrap(),
            &http_client
        ).await.unwrap();

        let client=
            CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(env::var("OIDC_CLIENT_ID").unwrap_or("5886b8495ece1e656e033bbb9c3aeb19e4f6175453fe40609ab7a34abed94bb5".to_string())),
            Some(ClientSecret::new(env::var("OIDC_CLIENT_SECRET").unwrap_or("gloas-0660c3c0f9289c4878a01e5a16b8a9796af5fee840ab6bacf85b379645816075".to_string()))),
        )
        .set_redirect_uri(RedirectUrl::new(env::var("OIDC_REDIRECT_URL").unwrap_or("http://localhost:5000".to_string())).unwrap());

        return Authenticator {
            http_client:  http_client,
            client: client
        }
    }
    
    pub fn get_redirect_url(&self) -> (reqwest::Url, CsrfToken, Nonce) {
        let (pkce_challenge, _pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        return self.client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            // Set the desired scopes.
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();
    }

    pub async fn get_from_redirected(&self, token: String, csrf: String) -> AccessToken {

        let pkce_verifier = PkceCodeVerifier::new(csrf);

        let token_response =self.client
        .exchange_code(AuthorizationCode::new(token)).unwrap()
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
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
        let auth_header = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|header| header.to_str().ok())
                .ok_or(StatusCode::UNAUTHORIZED)?;

            if let Some(user) = self.authorize(auth_header).await {
                req.extensions_mut().insert(user);
                return Ok(next.run(req).await);
            }

        return Err(StatusCode::UNAUTHORIZED);
    }
    
}