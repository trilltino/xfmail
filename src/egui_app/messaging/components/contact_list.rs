//! Contact List Component
//!
//! Displays the list of contacts with their last message preview.

use eframe::egui;
use uuid::Uuid;
use crate::egui_app::messaging::state::MessagingState;
use crate::egui_app::theme::colors;
use super::contact_item;

#[cfg(feature = "ssr")]
use chrono::Utc;

/// Render the contact list
pub fn render(ui: &mut egui::Ui, state: &mut MessagingState) {
    tracing::debug!("[BRAID] Rendering contact list, contacts: {}, conversations: {}", state.contacts.len(), state.conversations.len());

    // Collect contact data first to avoid borrow issues
    let contact_data: Vec<_> = {
        let filtered_contacts = state.filtered_contacts();

        if filtered_contacts.is_empty() {
            tracing::debug!("[BRAID] No filtered contacts to display");
            Vec::new()
        } else {
            filtered_contacts.iter().map(|contact| {
                let is_selected = state.selected_conversation_id
                    .map(|id| {
                        state.conversations.get(&id)
                            .map(|conv| conv.participants.contains(&contact.contact_user_id))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);

                // Find the conversation for this contact
                let conversation_id = state.conversations.values()
                    .find(|conv| conv.participants.contains(&contact.contact_user_id))
                    .map(|conv| conv.id);

                let last_message_content = conversation_id
                    .and_then(|id| state.messages.get(&id))
                    .and_then(|msgs| msgs.last())
                    .map(|msg| (msg.content.clone(), msg.timestamp.clone()));

                (
                    contact.contact_user_id,
                    contact.username.clone(),
                    contact.email.clone(),
                    contact.display_name.clone(),
                    is_selected,
                    conversation_id,
                    last_message_content,
                )
            }).collect()
        }
    };

    if contact_data.is_empty() {
        render_empty_state(ui, state);
    } else {
        let mut selected_conv: Option<Uuid> = None;

        for (contact_user_id, username, email, display_name, is_selected, conversation_id, last_message) in contact_data {
            // Create a temporary contact for rendering
            #[cfg(feature = "ssr")]
            let contact = crate::shared::messaging::Contact {
                id: Uuid::new_v4(), // Placeholder
                user_id: Uuid::new_v4(), // Placeholder
                contact_user_id,
                username,
                email,
                display_name,
                avatar_url: None,
                last_seen: Utc::now(),
                is_online: false,
                created_at: Utc::now(),
            };

            #[cfg(not(feature = "ssr"))]
            let contact = crate::shared::messaging::Contact {
                id: Uuid::new_v4(), // Placeholder
                user_id: Uuid::new_v4(), // Placeholder
                contact_user_id,
                username,
                email,
                display_name,
                avatar_url: None,
                last_seen: String::new(),
                is_online: false,
                created_at: String::new(),
            };

            // Create a temporary message for rendering if we have one
            let temp_message = last_message.map(|(content, timestamp)| {
                crate::shared::messaging::ChatMessage {
                    id: Uuid::new_v4(),
                    conversation_id: conversation_id.unwrap_or(Uuid::new_v4()),
                    sender_id: Uuid::new_v4(),
                    content,
                    message_type: crate::shared::messaging::MessageType::Text,
                    timestamp,
                    is_read: false,
                    is_delivered: true,
                    crdt_timestamp: 0,
                    braid_version: String::new(),
                    braid_parents: Vec::new(),
                    version_vector: crate::shared::messaging::message::VersionVector::default(),
                }
            });

            if contact_item::render(ui, &contact, temp_message.as_ref(), is_selected) {
                // Contact was clicked - select the conversation
                if let Some(conv_id) = conversation_id {
                    selected_conv = Some(conv_id);
                }
            }
        }

        // Apply selection after the loop
        if let Some(conv_id) = selected_conv {
            state.select_conversation(conv_id);
        }
    }
}

/// Render empty state when no contacts
fn render_empty_state(ui: &mut egui::Ui, state: &MessagingState) {
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);
        
        if state.search_query.is_empty() {
            ui.label("No contacts yet");
            ui.add_space(8.0);
            ui.colored_label(
                colors::TEXT_SECONDARY,
                "Add friends using the ➕ button above",
            );
        } else {
            ui.label("No contacts found");
            ui.add_space(8.0);
            ui.colored_label(
                colors::TEXT_SECONDARY,
                format!("No results for \"{}\"", state.search_query),
            );
        }
    });
}

