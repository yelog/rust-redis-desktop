use floem::{
    animate::Animation,
    peniko::Color,
    reactive::{create_rw_signal, RwSignal, SignalGet, SignalUpdate},
    style::{CursorStyle, Style},
    views::*,
    IntoView,
};

#[derive(Clone, Copy, PartialEq)]
enum ViewSwitcher {
    One,
    Two,
}

impl ViewSwitcher {
    fn toggle(&mut self) {
        *self = match self {
            ViewSwitcher::One => ViewSwitcher::Two,
            ViewSwitcher::Two => ViewSwitcher::One,
        };
    }

    fn view(&self, state: RwSignal<Self>) -> impl IntoView {
        match self {
            ViewSwitcher::One => view_one().into_any(),
            ViewSwitcher::Two => view_two(state).into_any(),
        }
        .animation(Animation::scale_size_effect)
        .clip()
    }
}

fn main() {
    floem::launch(app_view);
}

fn app_view() -> impl IntoView {
    let view = create_rw_signal(ViewSwitcher::One);
    let start_text = create_rw_signal("".to_string());
    let primary_color = Color::rgb8(0x16, 0x77, 0xFF);

    let hover_bg_color = Color::rgb8(0x40, 0x96, 0xFF); // 悬停时背景色

    h_stack((v_stack((
        button("New Connection")
            .action(move || view.update(|which| which.toggle()))
            .style(move |_cx| {
                Style::default()
                    .background(primary_color) // 正常背景色
                    .color(Color::WHITE) // 字体颜色
                    .height(32.0)
                    .width_full()
                    .font_size(14.0)
                    .padding_horiz(15.0)
                    .padding_vert(4.0)
                    .cursor(CursorStyle::Pointer)
                    .border(0)
                    .hover(|_cx| {
                        Style::default().background(hover_bg_color) // 悬停时背景色
                    })
            }),
        text_input(start_text)
            .placeholder("Search Connection")
            .style(|s| s.width_full()),
        dyn_container(move || view.get(), move |which| which.view(view))
            .style(|s| s.border(1.0).border_radius(5.0)),
    ))
    .style(|s| {
        s.width(200.0) // 设置 v_stack 的宽度为 200
            .height_full()
            .padding(10.0) // 添加填充以避免被遮挡
            .items_center()
            // .justify_center()
            .gap(20.0)
            .border_right(1.0)
            .border_color(Color::BLACK.multiply_alpha(0.2))
    }),))
    .style(|s| s.width_full().height_full().gap(20.0))
}

fn view_one() -> impl IntoView {
    // container used to make the text clip evenly on both sides while animating
    container("A view").style(|s| s.size(100.0, 100.0).items_center().justify_center())
}

fn view_two(view: RwSignal<ViewSwitcher>) -> impl IntoView {
    v_stack((
        "Another view".into_view(),
        button("Switch back").action(move || view.set(ViewSwitcher::One)),
    ))
    .style(|s| {
        s.column_gap(10.0)
            .size(150.0, 100.0)
            .items_center()
            .justify_center()
    })
}
