fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(FromApp)]
struct Root {
    heading: Glob<Text2D>,
    name_label: Glob<Text2D>,
    name_text_box: Glob<TextBox>,
    age_slider: Glob<Slider>,
    age_text_box: Glob<TextBox>,
    age_label: Glob<Text2D>,
    name: String,
    age: u32,
}

impl Singleton for Root {
    fn update(&mut self, app: &mut App) {
        let layout = VerticalLayout::new(5)
            .with_position(Vec2::ZERO)
            .with_size(Vec2::ONE);
        self.update_heading(app, layout.next(1.5));
        self.update_name_section(app, layout.next(1.));
        self.update_age_section(app, layout.next(1.));
    }
}

impl Root {
    fn update_heading(&mut self, app: &mut App, cell: Cell) {
        self.heading
            .updater()
            .with(Self::heading_text_style)
            .cell(cell)
            .label("My Modor Application")
            .apply(app);
    }

    fn update_name_section(&mut self, app: &mut App, cell: Cell) {
        let layout = VerticalLayout::new(2).with_cell(cell);
        self.name_label
            .updater()
            .with(style::dark::text)
            .cell(layout.next(0.75))
            .label("Your name:")
            .apply(app);
        self.name_text_box
            .updater()
            .style(style::dark::text_box)
            .cell(layout.next(1.))
            .for_value(app, |value| self.name = value.clone())
            .apply(app);
    }

    fn update_age_section(&mut self, app: &mut App, cell: Cell) {
        let layout = VerticalLayout::new(3).with_cell(cell);
        self.age_slider
            .updater()
            .style(style::dark::slider)
            .cell(layout.next(1.2))
            .range(0..=120)
            .for_value(app, |value| self.age = *value)
            .apply(app);
        self.age_text_box
            .updater()
            .style(style::dark::text_box)
            .cell(layout.next(1.))
            .type_(TextBoxType::Number)
            .disabled(true)
            .value(self.age)
            .apply(app);
        self.age_label
            .updater()
            .style(style::dark::text)
            .label("age")
            .cell(layout.next(1.))
            .apply(app);
    }

    fn heading_text_style(mut text: Text2DUpdater) -> Text2DUpdater {
        style::text::dark(text).color(Color::RED)
    }
}
