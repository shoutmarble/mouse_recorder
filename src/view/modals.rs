use super::*;

impl App {
    pub(crate) fn view_find_target_modal<'a>(&'a self, draft: &'a FindTargetDraft) -> Element<'a, Message> {
        let overlay_bg = container(iced::widget::Space::new())
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.55))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
            .width(Length::Fill)
            .height(Length::Fill);

        let have_image = draft.patch_png_base64.is_some();
        let image_status = if have_image {
            "Image: loaded"
        } else if draft.capture_waiting {
            "Click anywhere to capture…"
        } else {
            "Image: (none)"
        };

        let image_preview: Element<Message> = if let Some(b64) = draft.patch_png_base64.as_deref() {
            if let Some(handle) = self.thumb_handle_from_base64(b64) {
                container(image(handle).width(Length::Fixed(96.0)).height(Length::Fixed(96.0)))
                    .padding(6)
                    .style(|_| iced::widget::container::Style {
                        text_color: None,
                        background: Some(Background::Color(Color::from_rgb8(0x18, 0x1b, 0x1f))),
                        border: Border {
                            color: Color::from_rgb8(0x3a, 0x3f, 0x46),
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        shadow: Shadow::default(),
                        snap: false,
                    })
                    .into()
            } else {
                text("(preview failed)").size(14).into()
            }
        } else {
            container(iced::widget::Space::new())
                .width(Length::Fixed(96.0))
                .height(Length::Fixed(96.0))
                .into()
        };

        let controls = iced::widget::column![
            text("Find target (move only)").size(18),
            text("Find target and move there; click events come from the main timeline rows.")
                .size(13),
            text(image_status).size(14),
            image_preview,
            text(&draft.status).size(14),
            row![
                button(text("Capture from screen"))
                    .on_press(Message::FindTargetStartCapture),
                button(text("Load from path"))
                    .on_press(Message::FindTargetLoadFromPath),
            ]
            .spacing(10),
            row![
                text("Path:").size(14),
                text_input("template.png", &draft.image_path)
                    .on_input(Message::FindTargetPathChanged)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_y(alignment::Alignment::Center),
            row![
                text("Anchor:").size(14),
                pick_list(
                    [
                        SearchAnchor::RecordedClick,
                        SearchAnchor::CurrentMouse,
                        SearchAnchor::LastFound,
                    ],
                    Some(draft.anchor),
                    Message::FindTargetAnchorSelected,
                )
                .width(Length::Fixed(140.0)),
            ]
            .spacing(10)
            .align_y(alignment::Alignment::Center),
            row![
                text("Patch:").size(14),
                text_input("64", &draft.patch_size_text)
                    .on_input(Message::FindTargetPatchSizeChanged)
                    .width(Length::Fixed(70.0)),
                text("px").size(14),
                text("Prec:").size(14),
                text_input("0.92", &draft.precision_text)
                    .on_input(Message::FindTargetPrecisionChanged)
                    .width(Length::Fixed(70.0)),
            ]
            .spacing(10)
            .align_y(alignment::Alignment::Center),
            row![
                text("Timeout:").size(14),
                text_input("2000", &draft.timeout_ms_text)
                    .on_input(Message::FindTargetTimeoutChanged)
                    .width(Length::Fixed(80.0)),
                text("ms").size(14),
            ]
            .spacing(10)
            .align_y(alignment::Alignment::Center),
            row![
                checkbox(draft.limit_region)
                    .label("Limit region")
                    .on_toggle(Message::FindTargetLimitRegionToggled),
                text("Region:").size(14),
                text_input("600", &draft.region_size_text)
                    .on_input(Message::FindTargetRegionSizeChanged)
                    .width(Length::Fixed(80.0)),
                text("px").size(14),
            ]
            .spacing(10)
            .align_y(alignment::Alignment::Center),
            row![
                button(text("Cancel")).on_press(Message::CloseModal),
                button(text("OK (add row)"))
                    .on_press_maybe(have_image.then_some(Message::FindTargetOk)),
            ]
            .spacing(10),
        ]
        .spacing(10)
        .padding(16);

        let panel = container(controls)
            .width(Length::Fixed(560.0))
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(0x22, 0x25, 0x2a))),
                border: Border {
                    color: Color::from_rgb8(0x3a, 0x3f, 0x46),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba8(0, 0, 0, 0.35),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                snap: false,
            });

        let centered_panel = container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center);

        stack![overlay_bg, centered_panel].into()
    }

    pub(crate) fn view_wait_modal<'a>(&'a self, draft: &'a WaitDraft) -> Element<'a, Message> {
        let overlay_bg = container(iced::widget::Space::new())
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.55))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
            .width(Length::Fill)
            .height(Length::Fill);

        let controls = iced::widget::column![
            text("Wait").size(18),
            text("Add a fixed delay row.")
                .size(14),
            text(&draft.status).size(14),
            row![
                text("Wait:").size(14),
                text_input("1000", &draft.wait_ms_text)
                    .on_input(Message::WaitMsChanged)
                    .width(Length::Fixed(100.0)),
                text("ms").size(14),
            ]
            .spacing(10)
            .align_y(alignment::Alignment::Center),
            row![
                button(text("Cancel")).on_press(Message::CloseModal),
                button(text("OK (add row)")).on_press(Message::WaitOk),
            ]
            .spacing(10),
        ]
        .spacing(10)
        .padding(16);

        let panel = container(controls)
            .width(Length::Fixed(520.0))
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(0x22, 0x25, 0x2a))),
                border: Border {
                    color: Color::from_rgb8(0x3a, 0x3f, 0x46),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba8(0, 0, 0, 0.35),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                snap: false,
            });

        let centered_panel = container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center);

        stack![overlay_bg, centered_panel].into()
    }
}
