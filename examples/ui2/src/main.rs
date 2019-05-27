use cgmath::{Point2, Vector2};

use good_web_game::{
    event,
    goodies::megaui::{widgets::*, Drag, Ui},
    hash, Context, GameResult,
};

pub struct Slot {
    id: u64,
    item: Option<String>,
}
impl Slot {
    fn new(id: u64) -> Slot {
        Slot { id, item: None }
    }
}

pub struct Data {
    inventory: Vec<String>,
    item_dragging: bool,
    slots: Vec<(&'static str, Slot)>,
    windows: [bool; 20],
}
impl Data {
    pub fn new() -> Data {
        Data {
            inventory: vec![],
            item_dragging: false,
            slots: vec![
                ("Left Mouse Button", Slot::new(hash!())),
                ("Right Mouse Button", Slot::new(hash!())),
                ("Middle Mouse Button", Slot::new(hash!())),
                ("Space", Slot::new(hash!())),
                ("\"1\"", Slot::new(hash!())),
                ("\"2\"", Slot::new(hash!())),
                ("\"3\"", Slot::new(hash!())),
            ],
            windows: [true; 20],
        }
    }

    fn slots(&mut self, ui: &mut Ui) {
        let item_dragging = &mut self.item_dragging;

        for (label, item) in self.slots.iter_mut() {
            Group::new(hash!("grp", item.id, &label), Vector2::new(210., 55.)).ui(ui, |ui| {
                let drag = Group::new(item.id, Vector2::new(50., 50.))
                    .draggable(true)
                    .highlight(*item_dragging)
                    .ui(ui, |ui| {
                        if let Some(ref item) = item.item {
                            ui.label(Point2::new(5., 10.), &item);
                        }
                    });

                match drag {
                    Drag::Dropped(_, id) => {
                        if id.map_or(true, |id| id != item.id) {
                            item.item = None;
                        }
                        *item_dragging = false;
                    }
                    Drag::Dragging => {
                        *item_dragging = true;
                    }
                    Drag::No => {}
                }
                ui.label(Point2::new(60., 20.), label);
            });
        }
    }

    fn inventory(&mut self, ui: &mut Ui) {
        let item_dragging = &mut self.item_dragging;
        for (n, item) in self.inventory.iter().enumerate() {
            let drag = Group::new(hash!("inventory", n), Vector2::new(50., 50.))
                .draggable(true)
                .ui(ui, |ui| {
                    ui.label(Point2::new(5., 10.), &item);
                });

            match drag {
                Drag::Dropped(_, Some(id)) => {
                    for slot in self.slots.iter_mut() {
                        if slot.1.id == id {
                            slot.1.item = Some(item.to_string());
                        }
                    }
                    *item_dragging = false;
                }
                Drag::Dropped { .. } => {
                    *item_dragging = false;
                }
                Drag::Dragging => {
                    *item_dragging = true;
                }
                _ => {}
            }
        }
    }
}
pub struct MainState {
    data: Data,
    ui: Ui,
}

impl MainState {
    pub fn new() -> MainState {
        MainState {
            data: Data::new(),
            ui: Ui::new(),
        }
    }
}

impl MainState {}

impl event::EventHandler for MainState {
    fn key_down_event(&mut self, _: &mut Context, _: &str) {}

    fn mouse_button_down_event(
        &mut self,
        _: &mut Context,
        _: good_web_game::input::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.ui.mouse_down(cgmath::Point2::new(x, y));
    }

    fn mouse_button_up_event(
        &mut self,
        _: &mut Context,
        _: good_web_game::input::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.ui.mouse_up(cgmath::Point2::new(x, y));
    }

    fn mouse_wheel_event(&mut self, _: &mut Context, x: f32, y: f32) {
        self.ui.mouse_wheel(x, y);
    }

    fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32, _: f32, _: f32) {
        self.ui.mouse_move(cgmath::Point2::new(x, y));
    }

    fn update(&mut self, _: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> good_web_game::GameResult<()> {
        good_web_game::graphics::clear(ctx, [1., 1., 1., 1.].into());

        self.ui.begin_frame();

        let mut ui = &mut self.ui;
        let mut data = &mut self.data;
        Window::new(hash!(), Point2::new(400., 200.), Vector2::new(320., 400.)).ui(&mut ui, |ui| {
            for i in 0..30 {
                Group::new(hash!("shop", i), Vector2::new(290., 80.)).ui(ui, |ui| {
                    ui.label(Point2::new(10., 10.), &format!("Item N {}", i));
                    ui.label(Point2::new(260., 40.), "10/10");
                    ui.label(Point2::new(200., 63.), &format!("{} kr", 800));
                    if ui.button(Point2::new(260., 60.), hash!(i), "buy") {
                        data.inventory.push(format!("Item {}", i));
                    }
                });
            }
        });

        Window::new(hash!(), Point2::new(100., 220.), Vector2::new(512., 420.)).ui(&mut ui, |ui| {
            Group::new(hash!(), Vector2::new(220., 400.)).ui(ui, |ui| {
                data.slots(ui);
            });
            Group::new(hash!(), Vector2::new(280., 400.)).ui(ui, |ui| {
                data.inventory(ui);
            });
        });

        for i in 0..data.windows.len() {
            if data.windows[i] {
                data.windows[i] = Window::new(
                    hash!(i, "closebug"),
                    Point2::new(400., 720.),
                    Vector2::new(100., 100.),
                )
                .close_button(true)
                .ui(&mut self.ui, |ui| {
                    ui.label(None, "close button bug");
                });
            }
        }

        self.ui.draw(ctx);

        Ok(())
    }

    fn resize_event(&mut self, _: &mut good_web_game::Context, _: f32, _: f32) {}
}

fn main() -> GameResult {
    use good_web_game::conf;

    good_web_game::start(
        conf::Conf {
            cache: conf::Cache::No,
            ..Default::default()
        },
        |context| {
            let state = MainState::new();
            event::run(context, state)
        },
    )
}
