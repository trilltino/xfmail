//! Friend Request API Client
//!
//! This module provides async functions for interacting with the friend request API.

use crate::egui_app::config::Config;
use crate::shared::messaging::{
    Contact, Conversation, FriendRequest, ListContactsResponse, ListConversationsResponse,
    ListFriendRequestsResponse, RespondFriendRequestRequest, RespondFriendRequestResponse,
    SendFriendRequestRequest, SendFriendRequestResponse,
};
use reqwest::Client;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Friend API client
pub struct FriendApiClient {
    config: Config,
    client: Client,
}

impl FriendApiClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Send a friend request to a user by email
    pub fn send_friend_request(&self, to_email: &str) -> Result<SendFriendRequestResponse, String> {
        let url = self.config.api_url("/api/friends/request");
        let token = self.config.get_token().ok_or("Not authenticated")?;

        let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            let request = SendFriendRequestRequest {
                to_email: to_email.to_string(),
            };

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| status.to_string());
                
                let friendly_error = match status.as_u16() {
                    409 => "Friend request already sent or already friends".to_string(),
                    404 => "User not found".to_string(),
                    _ => format!("Request failed: {} - {}", status, error_text),
                };
                return Err(friendly_error);
            }

            response
                .json::<SendFriendRequestResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        })
    }

    /// Get pending friend requests for the current user
    pub fn get_pending_requests(&self) -> Result<Vec<FriendRequest>, String> {
        let url = self.config.api_url("/api/friends/requests");
        let token = self.config.get_token().ok_or("Not authenticated")?;

        let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| status.to_string());
                return Err(format!("Request failed: {} - {}", status, error_text));
            }

            let list_response = response
                .json::<ListFriendRequestsResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok(list_response.requests)
        })
    }

    /// Respond to a friend request (accept or reject)
    pub fn respond_to_request(
        &self,
        request_id: Uuid,
        accept: bool,
    ) -> Result<RespondFriendRequestResponse, String> {
        let url = self.config.api_url("/api/friends/respond");
        let token = self.config.get_token().ok_or("Not authenticated")?;

        let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            let request = RespondFriendRequestRequest { request_id, accept };

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| status.to_string());
                return Err(format!("Request failed: {} - {}", status, error_text));
            }

            response
                .json::<RespondFriendRequestResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        })
    }

    /// Get contacts for the current user
    pub fn get_contacts(&self) -> Result<Vec<Contact>, String> {
        let url = self.config.api_url("/api/contacts");
        let token = self.config.get_token().ok_or("Not authenticated")?;

        let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| status.to_string());
                return Err(format!("Request failed: {} - {}", status, error_text));
            }

            let list_response = response
                .json::<ListContactsResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok(list_response.contacts)
        })
    }

    /// Get conversations for the current user
    pub fn get_conversations(&self) -> Result<Vec<Conversation>, String> {
        let url = self.config.api_url("/api/conversations");
        let token = self.config.get_token().ok_or("Not authenticated")?;

        let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| status.to_string());
                return Err(format!("Request failed: {} - {}", status, error_text));
            }

            let list_response = response
                .json::<ListConversationsResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok(list_response.conversations)
        })
    }
}

