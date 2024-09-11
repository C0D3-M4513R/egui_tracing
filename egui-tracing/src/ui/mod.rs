mod color;
mod components;
mod state;

use std::sync::{Arc, Mutex};

use egui::{Color32, Response, TextStyle, TextWrapMode, Widget};

use self::color::ToColor32;
use self::components::constants;
use self::components::level_menu_button::LevelMenuButton;
use self::components::target_menu_button::TargetMenuButton;
use self::state::LogsState;
use crate::time::DateTimeFormatExt;
use crate::tracing::collector::EventCollector;

#[derive(Debug, Clone)]
pub struct Logs {
    collector: EventCollector,
    glob: Option<globset::GlobSet>,
}

impl Logs {
    #[must_use]
    pub const fn new(collector: EventCollector) -> Self {
        Self { collector, glob: None }
    }
}

impl Widget for Logs {
    fn ui(mut self, ui: &mut egui::Ui) -> Response {
        Widget::ui(&mut self, ui)
    }
}
impl Widget for &mut Logs {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let state = ui.memory_mut(|mem| {
            let state_mem_id = ui.id();
            mem.data
                .get_temp_mut_or_insert_with(state_mem_id, || {
                    Arc::new(Mutex::new(LogsState::default()))
                })
                .clone()
        });
        let mut state = state.lock().unwrap();

        let glob = self.glob.get_or_insert_with(||{
            let mut glob = globset::GlobSetBuilder::new();
            for target in state.target_filter.targets.clone() {
                glob.add(target);
            }
            glob.build().unwrap()
        });

        let events = self.collector.events();
        let filtered_events = events
            .iter()
            .rev()
            .filter(|event| state.level_filter.get(event.level) && !glob.is_match(&event.target))
            .collect::<Vec<_>>();

        let row_height = ui.style().text_styles.get(&TextStyle::Small).unwrap().size;

        ui.allocate_ui(egui::Vec2::new(100.+80.+100.+120.+120., (row_height + constants::SEPARATOR_SPACING) * (filtered_events.len() as f32)), |ui|{
            egui_extras::TableBuilder::new(ui)
                .column(egui_extras::Column::initial(100.).resizable(true))
                .column(egui_extras::Column::initial(80.).resizable(true))
                .columns(egui_extras::Column::initial(120.).resizable(true),2)
                .striped(true)
                .vscroll(true)
                .header(row_height, |mut row|{
                    row.col(|ui|{
                        ui.set_min_width(100.);
                        ui.label("Time");
                    });
                    row.col(|ui|{
                        ui.set_min_width(80.);
                        LevelMenuButton::default()
                            .state(&mut state.level_filter)
                            .show(ui)
                    });
                    row.col(|ui|{
                        ui.set_min_width(120.);
                        TargetMenuButton::default()
                            .state(&mut state.target_filter)
                            .show(ui)
                    });
                    row.col(|ui|{
                        ui.set_min_width(120.);
                        ui.label("Message");
                    });
                }).body(|body|{
                    body.rows(row_height, filtered_events.len(), |mut row| {
                        match filtered_events.get(row.index()) {
                            None => {
                                for _ in 0..5 {
                                    row.col(|ui|{
                                        ui.label("Out of bounds index");
                                    });
                                }
                            }
                            Some(event) => {
                                row.col(|ui|{
                                    ui.add(egui::Label::new(egui::RichText::new(event.time.format_short()).color(Color32::GRAY)))
                                        .on_hover_text(event.time.format_detailed());
                                });
                                row.col(|ui|{
                                    ui.add(egui::Label::new(egui::RichText::new(event.level.as_str()).color(event.level.to_color32())));
                                });
                                row.col(|ui|{
                                    ui.add(egui::Label::new(egui::RichText::new(&event.target).color(Color32::GRAY)).wrap_mode(TextWrapMode::Truncate)).on_hover_text(&event.target);
                                });
                                row.col(|ui|{
                                    match event.fields.get("message") {
                                        Some(message) => {
                                            ui.add(egui::Label::new(egui::RichText::new(message).color(Color32::WHITE)).wrap_mode(TextWrapMode::Truncate))
                                                .on_hover_text(message);
                                        },
                                        None => {
                                            ui.add(egui::Label::new(egui::RichText::new("No Message available").color(Color32::GRAY)).wrap_mode(TextWrapMode::Truncate));
                                        },
                                    };
                                });
                            }
                        }
                    }
                )
            });
        }).response
    }
}
