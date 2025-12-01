#[cfg(feature = "ssr")]
pub mod api;

#[cfg(feature = "ssr")]
pub struct SubscriptionManager;

#[cfg(feature = "ssr")]
impl SubscriptionManager {
    pub fn new() -> Self {
        Self
    }
}
