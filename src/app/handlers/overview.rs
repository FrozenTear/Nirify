//! Overview settings message handler

use crate::config::models::WorkspaceShadow;
use crate::config::SettingsCategory;
use crate::messages::{Message, OverviewMessage as M};
use crate::types::Color;
use iced::Task;

impl super::super::App {
    /// Updates overview settings (workspace overview / exposé)
    pub(in crate::app) fn update_overview(&mut self, msg: M) -> Task<Message> {
        match msg {
            M::SetZoom(zoom) => {
                self.settings.overview.zoom = zoom.clamp(0.1, 2.0);
                log::info!("Set overview zoom to {:.2}", zoom);
            }

            M::SetBackdropColor(color_hex) => {
                self.settings.overview.backdrop_color = color_hex.and_then(|h| Color::from_hex(&h));
                log::info!("Set overview backdrop color");
            }

            M::ToggleWorkspaceShadow(enabled) => {
                if enabled {
                    // Enable with defaults if currently None
                    if self.settings.overview.workspace_shadow.is_none() {
                        self.settings.overview.workspace_shadow = Some(WorkspaceShadow::default());
                    } else if let Some(ref mut shadow) = self.settings.overview.workspace_shadow {
                        shadow.enabled = true;
                    }
                } else {
                    // Disable by setting enabled = false or clearing
                    if let Some(ref mut shadow) = self.settings.overview.workspace_shadow {
                        shadow.enabled = false;
                    }
                }
                log::info!("Set overview workspace shadow enabled: {}", enabled);
            }

            M::SetWorkspaceShadowSoftness(softness) => {
                if let Some(ref mut shadow) = self.settings.overview.workspace_shadow {
                    shadow.softness = softness.clamp(0, 200);
                }
                log::info!("Set workspace shadow softness to {}", softness);
            }

            M::SetWorkspaceShadowSpread(spread) => {
                if let Some(ref mut shadow) = self.settings.overview.workspace_shadow {
                    shadow.spread = spread.clamp(0, 200);
                }
                log::info!("Set workspace shadow spread to {}", spread);
            }

            M::SetWorkspaceShadowOffsetX(offset_x) => {
                if let Some(ref mut shadow) = self.settings.overview.workspace_shadow {
                    shadow.offset_x = offset_x.clamp(-100, 100);
                }
                log::info!("Set workspace shadow offset X to {}", offset_x);
            }

            M::SetWorkspaceShadowOffsetY(offset_y) => {
                if let Some(ref mut shadow) = self.settings.overview.workspace_shadow {
                    shadow.offset_y = offset_y.clamp(-100, 100);
                }
                log::info!("Set workspace shadow offset Y to {}", offset_y);
            }

            M::SetWorkspaceShadowColor(color_hex) => {
                if let Some(ref mut shadow) = self.settings.overview.workspace_shadow {
                    if let Some(color) = Color::from_hex(&color_hex) {
                        shadow.color = color;
                    }
                }
                log::info!("Set workspace shadow color");
            }
        }

        self.save.dirty_tracker.mark(SettingsCategory::Overview);
        self.mark_changed();

        Task::none()
    }
}
