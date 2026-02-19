use super::*;

fn speed_slider_style(is_inactive: bool) -> iced::widget::slider::Style {
    let active_blue = Color::from_rgb8(0x34, 0x88, 0xff);
    let inactive_gray = Color::from_rgb8(0x7b, 0x86, 0x95);
    let rail_color = if is_inactive {
        inactive_gray
    } else {
        active_blue
    };

    iced::widget::slider::Style {
        rail: iced::widget::slider::Rail {
            backgrounds: (Background::Color(rail_color), Background::Color(rail_color)),
            width: 4.0,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
        },
        handle: iced::widget::slider::Handle {
            shape: iced::widget::slider::HandleShape::Circle { radius: 8.0 },
            background: Background::Color(rail_color),
            border_width: 1.0,
            border_color: Color::from_rgb8(0xea, 0xee, 0xf4),
        },
    }
}

fn slider_tooltip_frame_style() -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(Color::from_rgb8(0xf2, 0xf6, 0xff)),
        background: Some(Background::Color(Color::from_rgb8(0x10, 0x14, 0x1b))),
        border: Border {
            color: Color::from_rgb8(0x68, 0x9d, 0xff),
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

impl App {
    pub(crate) fn view_mouse_path_panel(&self) -> Element<'_, Message> {
        const VALUE_COL_W: f32 = 62.0;

        let path_state_badge: Element<Message> = container(
            text(if self.recorder_mouse_path_enabled { "ON" } else { "OFF" }).size(11),
        )
        .padding([1, 8])
        .style(|_| iced::widget::container::Style {
            text_color: Some(if self.recorder_mouse_path_enabled {
                Color::from_rgb8(0xc8, 0xe6, 0xc9)
            } else {
                Color::from_rgb8(0xc7, 0xcf, 0xd9)
            }),
            background: Some(Background::Color(if self.recorder_mouse_path_enabled {
                Color::from_rgb8(0x1f, 0x3a, 0x2a)
            } else {
                Color::from_rgb8(0x2a, 0x2f, 0x36)
            })),
            border: Border {
                color: if self.recorder_mouse_path_enabled {
                    Color::from_rgb8(0x5b, 0x9f, 0x6b)
                } else {
                    Color::from_rgb8(0x6e, 0x7a, 0x89)
                },
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        })
        .into();

        let toggle_row = row![
            text("Mouse Path:").size(14).width(Length::Fixed(140.0)),
            path_state_badge,
            toggler(self.recorder_mouse_path_enabled)
                .on_toggle(Message::SetMousePathEnabled),
            container(iced::widget::Space::new()).width(Length::Fill),
        ]
        .spacing(8)
        .align_y(alignment::Alignment::Center)
        .width(Length::Fill);

        let speed_row = row![
            text("Mouse move speed:").size(14).width(Length::Fixed(140.0)),
            container(
                tooltip(
                    slider(
                        5..=500,
                        self.editor_mouse_move_speed_ms,
                        Message::EditorMouseMoveSpeedMsChanged,
                    )
                    .style(|_theme, _status| speed_slider_style(false))
                    .width(Length::Fill),
                    text(format!("{} ms", self.editor_mouse_move_speed_ms)),
                    TooltipPosition::Top,
                )
                .gap(6)
                .padding(8)
                .style(|_| slider_tooltip_frame_style()),
            )
            .width(Length::Fill),
            container(text(format!("{} ms", self.editor_mouse_move_speed_ms)).size(12))
                .width(Length::Fixed(VALUE_COL_W))
                .align_x(alignment::Horizontal::Right),
        ]
        .spacing(8)
        .align_y(alignment::Alignment::Center)
        .width(Length::Fill);

        let path_sampling_row = row![
            text("Path sampling:").size(14).width(Length::Fixed(140.0)),
            container(
                tooltip(
                    slider(
                        0..=10,
                        self.recorder_mouse_path_min_delta_px,
                        Message::MousePathMinDeltaPxChanged,
                    )
                    .style(|_theme, _status| speed_slider_style(false))
                    .width(Length::Fill),
                    text(format!("{} px", self.recorder_mouse_path_min_delta_px)),
                    TooltipPosition::Top,
                )
                .gap(6)
                .padding(8)
                .style(|_| slider_tooltip_frame_style()),
            )
            .width(Length::Fill),
            container(text(format!("{} px", self.recorder_mouse_path_min_delta_px)).size(12))
                .width(Length::Fixed(VALUE_COL_W))
                .align_x(alignment::Horizontal::Right),
        ]
        .spacing(8)
        .align_y(alignment::Alignment::Center)
        .width(Length::Fill);

        let wait_row = row![
            text("Wait:").size(14).width(Length::Fixed(140.0)),
            container(
                tooltip(
                    slider(0..=300, self.editor_wait_ms, Message::EditorWaitMsChanged)
                        .width(Length::Fill),
                    text(format!("{} ms", self.editor_wait_ms)),
                    TooltipPosition::Top,
                )
                .gap(6)
                .padding(8)
                .style(|_| slider_tooltip_frame_style()),
            )
            .width(Length::Fill),
            container(text(format!("{} ms", self.editor_wait_ms)).size(12))
                .width(Length::Fixed(VALUE_COL_W))
                .align_x(alignment::Horizontal::Right),
        ]
        .spacing(8)
        .align_y(alignment::Alignment::Center)
        .width(Length::Fill);

        let mouse_path_main_pane: Element<Message> = container(
            iced::widget::column![toggle_row]
                .spacing(8)
                .width(Length::Fill),
        )
        .padding(8)
        .width(Length::Fill)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x17, 0x1c, 0x23))),
            border: Border {
                color: Color::from_rgb8(0x6f, 0x7f, 0x93),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        })
        .into();

        let sampling_pane: Element<Message> = container(path_sampling_row)
            .padding(8)
            .width(Length::Fill)
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(0x12, 0x18, 0x20))),
                border: Border {
                    color: Color::from_rgb8(0x5d, 0x6d, 0x82),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
            .into();

        let speed_pane: Element<Message> = container(speed_row)
        .padding(8)
        .width(Length::Fill)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x12, 0x18, 0x20))),
            border: Border {
                color: Color::from_rgb8(0x5d, 0x6d, 0x82),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        })
        .into();

        let wait_pane: Element<Message> = container(wait_row)
        .padding(8)
        .width(Length::Fill)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x12, 0x18, 0x20))),
            border: Border {
                color: Color::from_rgb8(0x5d, 0x6d, 0x82),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        })
        .into();

        container(
            iced::widget::column![mouse_path_main_pane, speed_pane, sampling_pane, wait_pane]
                .spacing(8)
                .width(Length::Fill),
        )
        .padding(8)
        .width(Length::Fill)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x1c, 0x21, 0x28))),
            border: Border {
                color: Color::from_rgb8(0x76, 0x85, 0x96),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        })
        .into()
    }

    pub(crate) fn view_scale_panel(&self) -> Element<'_, Message> {
        let slider_and_ticks = iced::widget::column![
            tooltip(
                slider(25..=100, self.ui_scale_percent, Message::UiScaleChanged)
                    .width(Length::Fill),
                text(format!("{:.2}x", self.ui_scale_factor())),
                TooltipPosition::Top,
            )
            .gap(6)
            .padding(8)
            .style(|_| slider_tooltip_frame_style()),
            row![
                text("0.25").size(11),
                container(iced::widget::Space::new()).width(Length::Fill),
                text("0.5").size(11),
                container(iced::widget::Space::new()).width(Length::Fill),
                text("0.75").size(11),
                container(iced::widget::Space::new()).width(Length::Fill),
                text("1.0").size(11),
            ]
            .width(Length::Fill)
            .align_y(alignment::Alignment::Center),
        ]
        .spacing(2)
        .width(Length::Fill);

        let slider_row = row![
            text("Scale:").size(14).width(Length::Fixed(56.0)),
            slider_and_ticks,
            text(format!("{:.2}x", self.ui_scale_factor())).size(12),
        ]
        .spacing(8)
        .align_y(alignment::Alignment::Center);

        container(slider_row)
            .padding(10)
            .width(Length::Fill)
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(0x1f, 0x22, 0x26))),
                border: Border {
                    color: Color::from_rgb8(0x3a, 0x3f, 0x46),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
            .into()
    }

    pub(crate) fn view_click_editor_panel(&self) -> Element<'_, Message> {
        const PREVIEW_IMAGE_SIZE: f32 = 128.0;
        const PREVIEW_FRAME_PADDING: f32 = 2.0;
        const PREVIEW_FRAMED_SIZE: f32 = PREVIEW_IMAGE_SIZE + PREVIEW_FRAME_PADDING * 2.0;
        const PREVIEW_FRAME_RADIUS: f32 = 6.0;
        const PREVIEW_INNER_RADIUS: f32 = 0.0;
        const PREVIEW_TITLE_H: f32 = 34.0;
        const TARGET_COL_W: f32 = 140.0;
        const VALUE_COL_W: f32 = 62.0;

        let right_preview_border = if self.editor_capture_armed {
            Color::from_rgb8(0xe5, 0x39, 0x35)
        } else {
            Color::from_rgb8(0x76, 0x85, 0x96)
        };

        let fmt_xy = |pos: Option<(i32, i32)>| -> String {
            match pos {
                Some((x, y)) => format!("({x:>6}, {y:>6})"),
                None => "(     ?,      ?)".to_string(),
            }
        };

        let grabbed_xy_text = fmt_xy(match (
            self.editor_x_text.trim().parse::<i32>().ok(),
            self.editor_y_text.trim().parse::<i32>().ok(),
        ) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        });

        let capture_status_text = if self.editor_capture_armed {
            "Armed: click L/R/M"
        } else {
            match self.editor_last_capture_button {
                Some("Left") => "Last capture: Left",
                Some("Right") => "Last capture: Right",
                Some("Middle") => "Last capture: Middle",
                _ => "Last capture: —",
            }
        };

        let capture_status_color = if self.editor_capture_armed {
            Color::from_rgb8(0xff, 0xc1, 0x07)
        } else if self.editor_last_capture_button.is_some() {
            Color::from_rgb8(0x8b, 0xd4, 0x9b)
        } else {
            Color::from_rgb8(0xb6, 0xc1, 0xcf)
        };

        let target_mode_box = || -> Element<Message> {
            let xy_row = radio(
                "GOTO (X,Y)",
                false,
                Some(self.editor_use_find_image),
                Message::EditorUseFindImageToggled,
            );

            let image_row = radio(
                "FIND IMAGE",
                true,
                Some(self.editor_use_find_image),
                Message::EditorUseFindImageToggled,
            );

            container(
                iced::widget::column![xy_row, image_row]
                    .spacing(8)
                    .align_x(alignment::Alignment::Start),
            )
            .padding(8)
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(0x1c, 0x21, 0x28))),
                border: Border {
                    color: Color::from_rgb8(0x76, 0x85, 0x96),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
            .into()
        };

        let target_match_box = || -> Element<Message> {
            let precision_value = self.editor_target_precision_percent as f32 / 100.0;

            iced::widget::column![
                text("Image match").size(12).color(Color::from_rgb8(0xc0, 0xca, 0xd6)),
                container(
                    iced::widget::column![
                        tooltip(
                            text("Precision").size(12),
                            "Sets the match confidence threshold. Higher values are stricter and reduce false positives.",
                            TooltipPosition::Top,
                        )
                        .gap(6)
                        .padding(8)
                        .style(|_| iced::widget::container::Style {
                            text_color: Some(Color::from_rgb8(0xf2, 0xf6, 0xff)),
                            background: Some(Background::Color(Color::from_rgb8(0x10, 0x14, 0x1b))),
                            border: Border {
                                color: Color::from_rgb8(0x68, 0x9d, 0xff),
                                width: 1.0,
                                radius: 8.0.into(),
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        }),
                        row![
                            container(
                                tooltip(
                                    slider(
                                        50..=100,
                                        self.editor_target_precision_percent,
                                        Message::EditorTargetPrecisionChanged,
                                    )
                                    .width(Length::Fill),
                                    text(format!("{precision_value:.2}")),
                                    TooltipPosition::Top,
                                )
                                .gap(6)
                                .padding(8)
                                .style(|_| slider_tooltip_frame_style()),
                            )
                            .width(Length::Fill),
                            container(text(format!("{precision_value:.2}")).size(12))
                                .width(Length::Fixed(VALUE_COL_W))
                                .align_x(alignment::Horizontal::Right),
                        ]
                        .spacing(8)
                        .align_y(alignment::Alignment::Center)
                        .width(Length::Fill),
                    ]
                    .spacing(8),
                )
                .padding(8)
                .style(|_| iced::widget::container::Style {
                    text_color: None,
                    background: Some(Background::Color(Color::from_rgb8(0x1a, 0x1f, 0x26))),
                    border: Border {
                        color: Color::from_rgb8(0x76, 0x85, 0x96),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }),
                container(
                    iced::widget::column![
                        tooltip(
                            text("Timeout").size(12),
                            "Sets how long to search before this target click fails.",
                            TooltipPosition::Top,
                        )
                        .gap(6)
                        .padding(8)
                        .style(|_| iced::widget::container::Style {
                            text_color: Some(Color::from_rgb8(0xf2, 0xf6, 0xff)),
                            background: Some(Background::Color(Color::from_rgb8(0x10, 0x14, 0x1b))),
                            border: Border {
                                color: Color::from_rgb8(0x68, 0x9d, 0xff),
                                width: 1.0,
                                radius: 8.0.into(),
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        }),
                        row![
                            container(
                                tooltip(
                                    slider(
                                        200..=10000,
                                        self.editor_target_timeout_ms,
                                        Message::EditorTargetTimeoutMsChanged,
                                    )
                                    .width(Length::Fill),
                                    text(format!("{} ms", self.editor_target_timeout_ms)),
                                    TooltipPosition::Top,
                                )
                                .gap(6)
                                .padding(8)
                                .style(|_| slider_tooltip_frame_style()),
                            )
                            .width(Length::Fill),
                            container(text(format!("{} ms", self.editor_target_timeout_ms)).size(12))
                                .width(Length::Fixed(VALUE_COL_W))
                                .align_x(alignment::Horizontal::Right),
                        ]
                        .spacing(8)
                        .align_y(alignment::Alignment::Center)
                        .width(Length::Fill),
                    ]
                    .spacing(8),
                )
                .padding(8)
                .style(|_| iced::widget::container::Style {
                    text_color: None,
                    background: Some(Background::Color(Color::from_rgb8(0x1a, 0x1f, 0x26))),
                    border: Border {
                        color: Color::from_rgb8(0x76, 0x85, 0x96),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }),
            ]
            .spacing(8)
            .align_x(alignment::Alignment::Start)
            .into()
        };

        fn preview_column_frame<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
            container(content)
                .padding(8)
                .style(|_| iced::widget::container::Style {
                    text_color: None,
                    background: Some(Background::Color(Color::from_rgb8(0x1c, 0x21, 0x28))),
                    border: Border {
                        color: Color::from_rgb8(0x76, 0x85, 0x96),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                })
                .into()
        }

        let static_preview: Element<Message> = if let Some(b64) = self.editor_static_preview_patch_b64.as_deref() {
            if let Some(handle) = self.preview_handle_from_base64(b64) {
                container(
                    iced::widget::column![
                        container(text("Grabbed preview (GET X,Y)").size(12))
                            .height(Length::Fixed(PREVIEW_TITLE_H))
                            .align_y(alignment::Vertical::Center),
                        row![
                            preview_column_frame(
                                iced::widget::column![
                                    container(
                                        container(
                                            image(handle)
                                                .width(Length::Fixed(PREVIEW_IMAGE_SIZE))
                                                .height(Length::Fixed(PREVIEW_IMAGE_SIZE)),
                                        )
                                        .width(Length::Fixed(PREVIEW_IMAGE_SIZE))
                                        .height(Length::Fixed(PREVIEW_IMAGE_SIZE))
                                        .style(|_| iced::widget::container::Style {
                                            text_color: None,
                                            background: Some(Background::Color(Color::from_rgb8(0x18, 0x1b, 0x1f))),
                                            border: Border {
                                                color: Color::TRANSPARENT,
                                                width: 0.0,
                                                radius: PREVIEW_INNER_RADIUS.into(),
                                            },
                                            shadow: Shadow::default(),
                                            snap: false,
                                        }),
                                    )
                                    .padding(PREVIEW_FRAME_PADDING)
                                    .width(Length::Fixed(PREVIEW_FRAMED_SIZE))
                                    .height(Length::Fixed(PREVIEW_FRAMED_SIZE))
                                    .style(move |_| iced::widget::container::Style {
                                        text_color: None,
                                        background: Some(Background::Color(Color::from_rgb8(0x18, 0x1b, 0x1f))),
                                        border: Border {
                                            color: right_preview_border,
                                            width: 2.0,
                                            radius: PREVIEW_FRAME_RADIUS.into(),
                                        },
                                        shadow: Shadow::default(),
                                        snap: false,
                                    }),
                                    text(format!("GET:  {grabbed_xy_text}")).size(12),
                                ]
                                .spacing(4)
                                .into(),
                            ),
                            preview_column_frame(
                                iced::widget::column![
                                    text(capture_status_text).size(12).color(capture_status_color),
                                    button(text("GET (X,Y)"))
                                        .on_press(Message::EditorStartGetXY),
                                    target_mode_box(),
                                ]
                                .spacing(10)
                                .align_x(alignment::Alignment::Start)
                                .into(),
                            ),
                            preview_column_frame(target_match_box()),
                        ]
                        .spacing(16)
                        .align_y(alignment::Alignment::Start)
                        .width(Length::Fill),
                    ]
                    .spacing(4),
                )
                .width(Length::FillPortion(1))
                .into()
            } else {
                container(
                    iced::widget::column![
                        container(text("Grabbed preview (GET X,Y)").size(12))
                            .height(Length::Fixed(PREVIEW_TITLE_H))
                            .align_y(alignment::Vertical::Center),
                        row![
                            preview_column_frame(
                                iced::widget::column![
                                    text("(static preview failed)").size(12),
                                    text(format!("GET:  {grabbed_xy_text}")).size(12),
                                ]
                                .spacing(4)
                                .into(),
                            ),
                            preview_column_frame(
                                iced::widget::column![
                                    text(capture_status_text).size(12).color(capture_status_color),
                                    button(text("GET (X,Y)"))
                                        .on_press(Message::EditorStartGetXY),
                                    target_mode_box(),
                                ]
                                .spacing(10)
                                .align_x(alignment::Alignment::Start)
                                .into(),
                            ),
                            preview_column_frame(target_match_box()),
                        ]
                        .spacing(16)
                        .align_y(alignment::Alignment::Start)
                        .width(Length::Fill),
                    ]
                    .spacing(4),
                )
                .width(Length::FillPortion(1))
                .into()
            }
        } else {
            container(
                iced::widget::column![
                    container(text("Grabbed preview (GET X,Y)").size(12))
                        .height(Length::Fixed(PREVIEW_TITLE_H))
                        .align_y(alignment::Vertical::Center),
                    row![
                        preview_column_frame(
                            iced::widget::column![
                                container(
                                    container(text("128 px x 128 px").size(14))
                                        .width(Length::Fixed(PREVIEW_IMAGE_SIZE))
                                        .height(Length::Fixed(PREVIEW_IMAGE_SIZE))
                                        .align_x(alignment::Horizontal::Center)
                                        .align_y(alignment::Vertical::Center)
                                        .style(|_| iced::widget::container::Style {
                                            text_color: None,
                                            background: Some(Background::Color(Color::from_rgb8(0x18, 0x1b, 0x1f))),
                                            border: Border {
                                                color: Color::TRANSPARENT,
                                                width: 0.0,
                                                radius: PREVIEW_INNER_RADIUS.into(),
                                            },
                                            shadow: Shadow::default(),
                                            snap: false,
                                        }),
                                )
                                .padding(PREVIEW_FRAME_PADDING)
                                .width(Length::Fixed(PREVIEW_FRAMED_SIZE))
                                .height(Length::Fixed(PREVIEW_FRAMED_SIZE))
                                .style(move |_| iced::widget::container::Style {
                                    text_color: None,
                                    background: Some(Background::Color(Color::from_rgb8(0x18, 0x1b, 0x1f))),
                                    border: Border {
                                        color: right_preview_border,
                                        width: 2.0,
                                        radius: PREVIEW_FRAME_RADIUS.into(),
                                    },
                                    shadow: Shadow::default(),
                                    snap: false,
                                }),
                                text(format!("GET:  {grabbed_xy_text}")).size(12),
                            ]
                            .spacing(4)
                            .into(),
                        ),
                        preview_column_frame(
                            iced::widget::column![
                                text(capture_status_text).size(12).color(capture_status_color),
                                button(text("GET (X,Y)"))
                                    .on_press(Message::EditorStartGetXY),
                                target_mode_box(),
                            ]
                            .spacing(10)
                            .align_x(alignment::Alignment::Start)
                            .into(),
                        ),
                        preview_column_frame(target_match_box()),
                    ]
                    .spacing(16)
                    .align_y(alignment::Alignment::Start)
                    .width(Length::Fill),
                ]
                .spacing(4),
            )
            .width(Length::FillPortion(1))
            .into()
        };

        let static_preview: Element<Message> = container(static_preview)
            .padding(6)
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(0x1b, 0x1f, 0x26))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
            .into();

        let left_row = self.view_click_target_modes_row(
            ClickTarget::Left,
            self.editor_left_mode,
            Message::EditorLeftModeSelected,
            TARGET_COL_W,
        );
        let right_row = self.view_click_target_modes_row(
            ClickTarget::Right,
            self.editor_right_mode,
            Message::EditorRightModeSelected,
            TARGET_COL_W,
        );
        let middle_row = self.view_click_target_modes_row(
            ClickTarget::Middle,
            self.editor_middle_mode,
            Message::EditorMiddleModeSelected,
            TARGET_COL_W,
        );

        let insert_btn_label = if self.selected_index.is_some() {
            "APPLY TO SELECTED"
        } else {
            "INSERT CLICK"
        };

        let threshold_group: Element<Message> = container(
            iced::widget::column![
                text("Click detection thresholds").size(13).color(Color::from_rgb8(0xc0, 0xca, 0xd6)),
                row![
                    container(text("Click speed:").size(13)).width(Length::Fixed(TARGET_COL_W)),
                    container(
                        tooltip(
                            slider(
                                0..=100,
                                self.editor_click_speed_ms,
                                Message::EditorClickSpeedMsChanged,
                            )
                            .style(|_theme, _status| speed_slider_style(false))
                            .width(Length::Fill),
                            text(format!("{} ms", self.editor_click_speed_ms)),
                            TooltipPosition::Top,
                        )
                        .gap(6)
                        .padding(8)
                        .style(|_| slider_tooltip_frame_style()),
                    )
                    .width(Length::Fill),
                    container(text(format!("{} ms", self.editor_click_speed_ms)).size(12))
                        .width(Length::Fixed(VALUE_COL_W))
                        .align_x(alignment::Horizontal::Right),
                ]
                .spacing(8)
                .align_y(alignment::Alignment::Center)
                .width(Length::Fill),
                row![
                    container(text("Pixel split:").size(13)).width(Length::Fixed(TARGET_COL_W)),
                    container(
                        tooltip(
                            slider(
                                0..=20,
                                self.editor_click_split_px,
                                Message::EditorClickSplitPxChanged,
                            )
                            .style(|_theme, _status| speed_slider_style(false))
                            .width(Length::Fill),
                            text(format!("{} px", self.editor_click_split_px)),
                            TooltipPosition::Top,
                        )
                        .gap(6)
                        .padding(8)
                        .style(|_| slider_tooltip_frame_style()),
                    )
                    .width(Length::Fill),
                    container(text(format!("{} px", self.editor_click_split_px)).size(12))
                        .width(Length::Fixed(VALUE_COL_W))
                        .align_x(alignment::Horizontal::Right),
                ]
                .spacing(8)
                .align_y(alignment::Alignment::Center)
                .width(Length::Fill),
                row![
                    container(text("Hold split:").size(13)).width(Length::Fixed(TARGET_COL_W)),
                    container(
                        tooltip(
                            slider(
                                0..=100,
                                self.editor_click_max_hold_ms,
                                Message::EditorClickMaxHoldMsChanged,
                            )
                            .style(|_theme, _status| speed_slider_style(false))
                            .width(Length::Fill),
                            text(format!("{} ms", self.editor_click_max_hold_ms)),
                            TooltipPosition::Top,
                        )
                        .gap(6)
                        .padding(8)
                        .style(|_| slider_tooltip_frame_style()),
                    )
                    .width(Length::Fill),
                    container(text(format!("{} ms", self.editor_click_max_hold_ms)).size(12))
                        .width(Length::Fixed(VALUE_COL_W))
                        .align_x(alignment::Horizontal::Right),
                ]
                .spacing(8)
                .align_y(alignment::Alignment::Center)
                .width(Length::Fill),
            ]
            .spacing(6)
            .width(Length::Fill),
        )
        .padding(8)
        .width(Length::Fill)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x1c, 0x21, 0x28))),
            border: Border {
                color: Color::from_rgb8(0x76, 0x85, 0x96),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        })
        .into();

        let mode_rows = iced::widget::column![
            row![
                container(text("Click target:").size(14).color(Color::from_rgb8(0xd2, 0xda, 0xe5))).width(Length::Fixed(TARGET_COL_W)),
                container(text("Click properties:").size(14).color(Color::from_rgb8(0xd2, 0xda, 0xe5))).width(Length::Fill),
            ]
            .align_y(alignment::Alignment::Center)
            .spacing(8)
            .width(Length::Fill),
            left_row,
            right_row,
            middle_row,
            threshold_group,
        ]
        .spacing(8)
        .width(Length::Fill);

        let mode_group: Element<Message> = container(mode_rows)
        .padding(8)
        .width(Length::Fill)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x1c, 0x21, 0x28))),
            border: Border {
                color: Color::from_rgb8(0x76, 0x85, 0x96),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        })
        .into();

        let mut button_items: Vec<Element<Message>> = vec![
            container(button(text(insert_btn_label)).on_press(Message::EditorInsertOrApply))
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .into(),
        ];

        if self.editor_capture_armed {
            button_items.push(
                text("Capture armed: click Left/Right/Middle to set X,Y (ESC cancels)")
                    .size(12)
                    .into(),
            );
        }

        let buttons_group: Element<Message> = container(
            column(button_items).spacing(if self.editor_capture_armed { 8 } else { 0 }),
        )
        .padding(6)
        .width(Length::Fill)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x1c, 0x20, 0x25))),
            border: Border {
                color: Color::from_rgb8(0x7f, 0x8e, 0x9f),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        })
        .into();

        let preview_group = container(
            row![static_preview]
                .spacing(10)
                .width(Length::Fill)
                .align_y(alignment::Alignment::Start),
        )
        .padding(8)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x22, 0x27, 0x2f))),
            border: Border {
                color: Color::from_rgb8(0x6c, 0x78, 0x8a),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        });

        let click_editor_content = iced::widget::column![
            text("Click editor").size(16),
            preview_group,
            mode_group,
            buttons_group,
        ]
        .spacing(6);

        container(click_editor_content)
            .padding(10)
            .width(Length::Fill)
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(0x15, 0x18, 0x1d))),
                border: Border {
                    color: Color::from_rgb8(0x8d, 0x9b, 0xb0),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
            .into()
    }

    pub(crate) fn view_properties_panel(&self) -> iced::widget::Container<'_, Message> {
        let body: Element<Message> = {
            let selection_note: Element<Message> = match self.selected_index {
                Some(index) => text(format!("Selected row: {}", index)).size(12).into(),
                None => text("Selected row: none").size(12).into(),
            };

            iced::widget::column![
                selection_note,
                self.view_mouse_path_panel(),
                self.view_click_editor_panel(),
            ]
            .spacing(8)
            .into()
        };

        let body_scroll_content = row![
            container(body).width(Length::Fill),
            container(iced::widget::Space::new()).width(Length::Fixed(8.0)),
        ]
        .width(Length::Fill);

        let body_scroll = scrollable(body_scroll_content)
            .height(Length::Fill);

        container(container(body_scroll).padding(12))
            .height(Length::Fill)
            .width(Length::Fill)
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(0x1f, 0x22, 0x26))),
                border: Border {
                    color: Color::from_rgb8(0x3a, 0x3f, 0x46),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
    }
}
