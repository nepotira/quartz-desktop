//! # Notification Center & 3D Stacks Subsystem
//!
//! Controls incoming system alerts and historical notifications:
//! - Triggered by clicking the Clock or swiping 2 fingers from the right screen edge.
//! - **Notification Stacks**: Alerts automatically cluster in 3D stacked cards grouped by application.
//! - Click-to-expand with spring rollout animation.
//! - Inline reply fields for messaging apps.
//! - Batch dismiss ("Clear All").

use qz_animation::{SpringAnimation, SpringParams};
use std::collections::HashMap;

/// An individual notification item.
#[derive(Debug, Clone, PartialEq)]
pub struct NotificationItem {
    /// Unique notification ID.
    pub id: u64,
    /// Originating application ID (e.g. "org.quartz.mail").
    pub app_id: String,
    /// Originating application display name (e.g. "Mail").
    pub app_name: String,
    /// Alert title text.
    pub title: String,
    /// Alert body message.
    pub message: String,
    /// Timestamp in epoch seconds.
    pub timestamp_secs: f64,
    /// Optional inline reply placeholder text.
    pub inline_reply_placeholder: Option<String>,
}

/// A 3D grouped stack of notifications belonging to one application.
#[derive(Debug, Clone, PartialEq)]
pub struct NotificationStack {
    /// Application ID owning this stack.
    pub app_id: String,
    /// Notifications contained in this stack (most recent first).
    pub items: Vec<NotificationItem>,
    /// True if the stack has been clicked and unrolled into flat list.
    pub is_expanded: bool,
    /// Animation spring controlling rollout expansion $[0.0, 1.0]$.
    pub expansion_spring: SpringAnimation,
}

impl NotificationStack {
    /// Creates a new stack containing an initial alert.
    pub fn new(initial_item: NotificationItem) -> Self {
        let app_id = initial_item.app_id.clone();
        Self {
            app_id,
            items: vec![initial_item],
            is_expanded: false,
            expansion_spring: SpringAnimation::new(SpringParams::window_open(), 0.0, 0.0, 0.0),
        }
    }

    /// Pushes a new alert to the front of this stack.
    pub fn push(&mut self, item: NotificationItem) {
        self.items.insert(0, item);
    }

    /// Toggles stack expansion.
    pub fn toggle_expand(&mut self) -> bool {
        self.is_expanded = !self.is_expanded;
        let target = if self.is_expanded { 1.0 } else { 0.0 };
        self.expansion_spring.retarget(target);
        self.is_expanded
    }

    /// Current unroll progress $[0.0, 1.0]$.
    pub fn unroll_progress(&self) -> f32 {
        self.expansion_spring.sample_at(self.expansion_spring.elapsed_time()).position.clamp(0.0, 1.0) as f32
    }
}

/// Notification Center coordinator managing all application stacks.
pub struct NotificationCenterManager {
    stacks: HashMap<String, NotificationStack>,
    is_panel_visible: bool,
    next_id: u64,
}

impl Default for NotificationCenterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationCenterManager {
    /// Creates an empty Notification Center coordinator.
    pub fn new() -> Self {
        Self {
            stacks: HashMap::new(),
            is_panel_visible: false,
            next_id: 1,
        }
    }

    /// Ingests a new notification alert.
    pub fn post_notification(
        &mut self,
        app_id: &str,
        app_name: &str,
        title: &str,
        message: &str,
        inline_reply_placeholder: Option<&str>,
        timestamp_secs: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let item = NotificationItem {
            id,
            app_id: app_id.to_string(),
            app_name: app_name.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            timestamp_secs,
            inline_reply_placeholder: inline_reply_placeholder.map(|s| s.to_string()),
        };

        if let Some(stack) = self.stacks.get_mut(app_id) {
            stack.push(item);
        } else {
            self.stacks.insert(app_id.to_string(), NotificationStack::new(item));
        }

        id
    }

    /// Dismisses all notifications ("Clear All").
    pub fn clear_all(&mut self) {
        self.stacks.clear();
    }

    /// Toggles visibility of the Notification Center side panel.
    pub fn toggle_panel_visibility(&mut self) -> bool {
        self.is_panel_visible = !self.is_panel_visible;
        self.is_panel_visible
    }

    /// Number of active notification stacks.
    #[inline]
    pub fn stack_count(&self) -> usize {
        self.stacks.len()
    }

    /// Total count of unread notifications across all stacks.
    pub fn total_notifications_count(&self) -> usize {
        self.stacks.values().map(|s| s.items.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_stacking_by_app() {
        let mut nc = NotificationCenterManager::new();

        // 2 notifications from Mail
        nc.post_notification("org.quartz.mail", "Mail", "New Email", "Subject 1", None, 1.0);
        nc.post_notification("org.quartz.mail", "Mail", "New Email", "Subject 2", None, 2.0);

        // 1 notification from Messages
        nc.post_notification("org.quartz.messages", "Messages", "New Message", "Hello", None, 3.0);

        // Should produce exactly 2 stacks
        assert_eq!(nc.stack_count(), 2);
        assert_eq!(nc.total_notifications_count(), 3);
    }

    #[test]
    fn test_notification_stack_expansion_toggle() {
        let item = NotificationItem {
            id: 1,
            app_id: "org.quartz.mail".to_string(),
            app_name: "Mail".to_string(),
            title: "Test".to_string(),
            message: "Msg".to_string(),
            timestamp_secs: 1.0,
            inline_reply_placeholder: None,
        };
        let mut stack = NotificationStack::new(item);
        assert!(!stack.is_expanded);

        assert!(stack.toggle_expand());
        assert!(stack.is_expanded);

        assert!(!stack.toggle_expand());
        assert!(!stack.is_expanded);
    }
}
