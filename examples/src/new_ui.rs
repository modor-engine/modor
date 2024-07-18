const SCOPE: Scope = Scope::new("root");
const HEADING_KEY: Key = SCOPE.key(0);
const NAME_LABEL_KEY: Key = SCOPE.key(1);
const NAME_TEXTBOX_KEY: Key = SCOPE.key(2);

fn main() {
    modor_graphics::run(Level::Info, root);
}

fn root(app: &mut App) {
    app.run(form);
}

fn form(app: &mut App, form: &mut Form) {
    let layout = VerticalLayout::new(5).position(Vec2::ZERO).size(Vec2::ONE);
    heading(app, layout.next(1.5));
    name_section(app, form, layout.next(1.));
    age_section(app, form, layout.next(1.));
}

fn heading(app: &mut App, cell: Cell) {
    Text2D::desc(app, HEADING_KEY)
        .with(heading_text_style)
        .cell(cell)
        .label("My Modor Application");
}

fn heading_text_style(mut text: Text2D) -> Text2D {
    style::dark(text).color(Color::RED)
}

fn name_section(app: &mut App, form: &mut Form, cell: Cell) {
    let layout = VerticalLayout::new(2).cell(cell);
    Text2D::desc(app, NAME_LABEL_KEY)
        .style(style::dark)
        .cell(layout.next(0.75))
        .label("Your name:");
    TextBox::desc(app, NAME_TEXTBOX_KEY)
        .style(style::dark)
        .cell(layout.next(1.))
        .for_value(|value| form.name = value.clone());
}

fn age_section(app: &mut App, form: &mut Form, cell: Cell) {
    let layout = VerticalLayout::new(3).cell(cell);
    Slider::desc(app, NAME_LABEL_KEY)
        .style(style::dark)
        .cell(layout.next(1.2))
        .range(0..=120)
        .for_value(|value| form.age = *value);
    TextBox::desc(app, NAME_TEXTBOX_KEY)
        .style(style::dark)
        .cell(layout.next(1.))
        .type_(TextBoxType::Number)
        .disabled(true)
        .value(form.age);
    Text2D::desc(app, NAME_LABEL_KEY)
        .style(style::dark)
        .label("age")
        .cell(layout.next(1.));
}

#[derive(Default)]
struct Form {
    name: String,
    age: u32,
}

// SCOPE.unique_child(app) can be used to generate a scope for dropdown items
// SCOPE.delete(app) will recursively delete all items

// TODO: finish based on https://github.com/emilk/egui?tab=readme-ov-file#why-immediate-mode
// TODO: do the same for pong
// TODO: do the same for test examples
// TODO: optimize new_rendering_2d ? (Model2D::for_each ?)
// TODO: maybe remove need of scope by storing object IDs in context singletons
