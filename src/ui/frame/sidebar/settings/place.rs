use crate::geometry::Rect;
use crate::input::TextField;
use crate::text::TextSystem;
use crate::ui::frame::FrameLayout;
use crate::ui::frame::editor::place_field;
use crate::ui::metrics::UiMetrics;

use super::chips::Choice;
use super::column::{Column, LINE_HEIGHT, SECTION_GAP};
use super::general::{general_height, general_page};
use super::hosts::connections_page;
use super::{
    Chip, HostsView, SettingControl, SettingsFrame, SettingsPage, SettingsRequest, SettingsTab,
};

const NO_HOSTS: HostsView<'static> = HostsView {
    hosts: &[],
    first: 0,
    form: None,
    changing: false,
};

impl FrameLayout {
    pub fn place_settings(&mut self, text: &mut TextSystem, request: &SettingsRequest) {
        let page_height = general_height(&self.metrics);
        let content = page_height + tabs_height(&self.metrics);
        let (popup, mut column) = self.open_popup(text, request.strings.settings, content);
        let tabs = settings_tabs(&mut column, request);
        column.y += column.metrics.px(SECTION_GAP);
        let page = settings_page(&mut column, request, page_height);
        self.settings = Some(SettingsFrame { popup, tabs, page });
    }

    pub fn place_command_field(&mut self, text: &mut TextSystem, field: &TextField) {
        let metrics = self.metrics;
        let Some(SettingsPage::General(page)) = self.settings.as_mut().map(|frame| &mut frame.page)
        else {
            return;
        };
        let line = page.command.line.rect;
        let anchor = Rect::from_edges(line.center().x, line.y, line.right(), line.bottom());
        page.command_field = Some(place_field(text, &metrics, anchor, line, field));
    }
}

fn tabs_height(metrics: &UiMetrics) -> f32 {
    metrics.px(LINE_HEIGHT) + metrics.px(SECTION_GAP)
}

fn settings_tabs(column: &mut Column, request: &SettingsRequest) -> [Chip; 2] {
    let strings = request.strings;
    let choice = |tab: SettingsTab| -> Choice {
        let label = match tab {
            SettingsTab::General => strings.general,
            SettingsTab::Connections => strings.connections_tab,
        };
        (label, SettingControl::Tab(tab), tab == request.tab)
    };
    column.tab_row(SettingsTab::ALL.map(choice))
}

fn settings_page(column: &mut Column, request: &SettingsRequest, height: f32) -> SettingsPage {
    let SettingsRequest {
        preferences,
        strings,
        folder,
        ..
    } = *request;
    match request.tab {
        SettingsTab::General => {
            let page = general_page(column, strings, preferences, folder);
            SettingsPage::General(Box::new(page))
        }
        SettingsTab::Connections => {
            let view = request.hosts.unwrap_or(NO_HOSTS);
            SettingsPage::Connections(Box::new(connections_page(column, strings, view, height)))
        }
    }
}
