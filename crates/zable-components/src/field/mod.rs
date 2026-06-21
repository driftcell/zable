use gpui::{AnyElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};
use gpui_component::{StyledExt, ThemeColor, h_flex, label::Label, v_flex};

/// A single labeled input row, using the library's `Label` for the
/// caption and a muted hint underneath.
///
/// Owned strings (`SharedString`) are used instead of borrows so that the
/// type satisfies `RenderOnce`'s `'static` requirement.
#[derive(IntoElement)]
pub struct Field {
    label: SharedString,
    hint: SharedString,
    input: AnyElement,
    theme: ThemeColor,
}

impl Field {
    pub fn new(
        label: impl Into<SharedString>,
        hint: impl Into<SharedString>,
        input: impl IntoElement,
        theme: ThemeColor,
    ) -> Self {
        Self {
            label: label.into(),
            hint: hint.into(),
            input: input.into_any_element(),
            theme,
        }
    }
}

impl RenderOnce for Field {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        v_flex()
            .gap_1()
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .justify_between()
                    .child(
                        Label::new(self.label)
                            .text_sm()
                            .font_medium()
                            .text_color(self.theme.foreground),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(self.theme.muted_foreground)
                            .child(self.hint),
                    ),
            )
            .child(self.input)
    }
}
