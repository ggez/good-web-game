use crate::{
    graphics::{Rect, RectAttr},
    Context,
};
use cgmath::{MetricSpace, Point2, Vector2};
use smart_default::SmartDefault;

use std::collections::HashMap;

type Id = u64;

#[macro_export]
macro_rules! hash {
    ($s:expr) => {{
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let id = $s;

        let mut s = DefaultHasher::new();
        id.hash(&mut s);
        s.finish()
    }};
    () => {{
        let id = concat!(file!(), line!(), column!());
        hash!(id)
    }};
    ($($s:expr),*) => {{
        let mut s: u128 = 0;
        $(s += hash!($s) as u128;)*
        hash!(s)
    }};

}

pub(self) enum Widget {
    Label {
        pos: Option<Point2<f32>>,
        label: String,
    },
    Button {
        pos: Option<Point2<f32>>,
        id: Id,
        label: String,
    },
    Group(widgets::Group),
    Window(Window),
}

pub mod widgets {
    use super::{Drag, Id, Ui, Widget};
    use cgmath::{Point2, Vector2};

    pub struct Window {
        id: Id,
        position: Point2<f32>,
        size: Vector2<f32>,
        label: Option<String>,
    }

    impl Window {
        pub fn new(id: Id, position: Point2<f32>, size: Vector2<f32>) -> Window {
            Window {
                id,
                position,
                size,
                label: None,
            }
        }

        pub fn label(self, label: &str) -> Window {
            Window {
                label: Some(label.to_string()),
                ..self
            }
        }

        pub fn ui<F: FnOnce(&mut Ui)>(self, ui: &mut Ui, f: F) {
            let new_window = super::Window {
                id: self.id,
                label: self.label.unwrap_or("".to_string()),
                rect: super::Rect::new(self.position.x, self.position.y, self.size.x, self.size.y),
            };
            let window_element_id = ui.tree.insert_window(new_window);
            ui.current_element = Some(window_element_id);

            f(ui)
        }
    }

    pub struct Group {
        pub(super) size: Vector2<f32>,
        pub(super) draggable: bool,
        pub(super) hoverable: bool,
        pub(super) highlight: bool,
        pub(super) id: Id,
    }
    impl Group {
        pub fn new(id: Id, size: Vector2<f32>) -> Group {
            Group {
                draggable: false,
                highlight: false,
                hoverable: false,
                id,
                size,
            }
        }

        pub fn draggable(self, draggable: bool) -> Group {
            Group { draggable, ..self }
        }

        pub fn hoverable(self, hoverable: bool) -> Group {
            Group { hoverable, ..self }
        }

        pub fn highlight(self, highlight: bool) -> Group {
            Group { highlight, ..self }
        }

        pub fn ui<F: FnOnce(&mut Ui)>(self, ui: &mut Ui, f: F) -> Drag {
            let self_id = self.id;
            let widget_id = ui
                .tree
                .insert_widget(ui.current_element.unwrap(), Widget::Group(self));
            let current_element = ui.current_element;
            ui.current_element = Some(widget_id);

            f(ui);

            ui.current_element = current_element;
            ui.events.previous_frame_drag(self_id)
        }
    }
}
impl Widget {
    fn unwrap_window(&self) -> &Window {
        match self {
            Widget::Window(window) => {
                hash!();
                window
            }
            _ => panic!("not a window"),
        }
    }

    fn unwrap_window_mut(&mut self) -> &mut Window {
        match self {
            Widget::Window(window) => window,
            _ => panic!("not a window"),
        }
    }

    fn is_window(&self) -> bool {
        match self {
            Widget::Window(_) => true,
            _ => false,
        }
    }

    fn group_id(&self) -> Option<Id> {
        match self {
            Widget::Group(widgets::Group { id, .. }) => Some(*id),
            _ => None,
        }
    }
}
struct Window {
    id: Id,
    label: String,
    rect: Rect,
}

impl Window {
    fn window_contains(&self, point: Point2<f32>) -> bool {
        self.rect.contains(point)
    }

    fn title_contains(&self, point: Point2<f32>) -> bool {
        Rect::new(self.rect.x, self.rect.y, self.rect.w, consts::TITLE_HEIGHT).contains(point)
    }
}

mod consts {
    pub const MARGIN: f32 = 2.;
    pub const TITLE_HEIGHT: f32 = MARGIN * 2. + 10.;
    pub const MARGIN_BUTTON: f32 = 3.;
    pub const SCROLL_WIDTH: f32 = 10.;
    pub const SCROLL_MULTIPLIER: f32 = 3.;

    pub const INACTIVE_TITLE: &'static str = "#6668";
    pub const FOCUSED_TITLE: &'static str = "#000f";
    pub const FOCUSED_TEXT: &'static str = "000f";
    pub const INACTIVE_TEXT: &'static str = "#6668";
    pub const WINDOW_BACKGROUND_FOCUSED: &'static str = "#eeef";
    pub const WINDOW_BACKGROUND_INACTIVE: &'static str = "#eee8";
    pub const WINDOW_BORDER_FOCUSED: &'static str = "#444f";
    pub const WINDOW_BORDER_INACTIVE: &'static str = "#6668";
    pub const GROUP_BORDER_FOCUSED_HOVERED: &'static str = "#2924";
    pub const GROUP_BORDER_FOCUSED: &'static str = "#2224";
    pub const GROUP_BORDER_FOCUSED_HIGHLIGHT: &'static str = "#22e7";
    pub const GROUP_BORDER_INACTIVE_HOVERED: &'static str = "#1812";
    pub const GROUP_BORDER_INACTIVE: &'static str = "#1112";
    pub const BUTTON_BACKGROUND_FOCUSED_CLICKED: &'static str = "#bbbe";
    pub const BUTTON_BACKGROUND_FOCUSED_HOVERED: &'static str = "#aaae";
    pub const BUTTON_BACKGROUND_FOCUSED: &'static str = "#ccce";
    pub const BUTTON_BACKGROUND_INACTIVE: &'static str = "#ccc8";
    pub const SCROLLBAR_BACKGROUND_FOCUSED_CLICKED: &'static str = "#aaae";
    pub const SCROLLBAR_BACKGROUND_FOCUSED_HOVERED: &'static str = "#aaae";
    pub const SCROLLBAR_BACKGROUND_FOCUSED: &'static str = "#ccce";
    pub const SCROLLBAR_BACKGROUND_INACTIVE: &'static str = "#ccc8";

    pub fn background(focused: bool) -> &'static str {
        if focused {
            WINDOW_BACKGROUND_FOCUSED
        } else {
            WINDOW_BACKGROUND_INACTIVE
        }
    }

    pub fn window_border(focused: bool) -> &'static str {
        if focused {
            WINDOW_BORDER_FOCUSED
        } else {
            WINDOW_BORDER_INACTIVE
        }
    }

    pub fn drag_border(focused: bool, hovered: bool, highlight: bool) -> &'static str {
        if focused {
            if hovered {
                GROUP_BORDER_FOCUSED_HOVERED
            } else {
                if highlight {
                    GROUP_BORDER_FOCUSED_HIGHLIGHT
                } else {
                    GROUP_BORDER_FOCUSED
                }
            }
        } else {
            if hovered {
                GROUP_BORDER_INACTIVE_HOVERED
            } else {
                GROUP_BORDER_INACTIVE
            }
        }
    }

    pub fn title(focused: bool) -> &'static str {
        if focused {
            FOCUSED_TITLE
        } else {
            INACTIVE_TITLE
        }
    }

    pub fn text(focused: bool) -> &'static str {
        if focused {
            FOCUSED_TEXT
        } else {
            INACTIVE_TEXT
        }
    }

    pub fn button_background(focused: bool, hovered: bool, clicked: bool) -> &'static str {
        if focused {
            if clicked {
                BUTTON_BACKGROUND_FOCUSED_CLICKED
            } else if hovered {
                BUTTON_BACKGROUND_FOCUSED_HOVERED
            } else {
                BUTTON_BACKGROUND_FOCUSED
            }
        } else {
            BUTTON_BACKGROUND_INACTIVE
        }
    }

    pub fn scroll_bar_handle(focused: bool, hovered: bool, clicked: bool) -> &'static str {
        if focused {
            if clicked {
                SCROLLBAR_BACKGROUND_FOCUSED_CLICKED
            } else if hovered {
                SCROLLBAR_BACKGROUND_FOCUSED_HOVERED
            } else {
                SCROLLBAR_BACKGROUND_FOCUSED
            }
        } else {
            SCROLLBAR_BACKGROUND_INACTIVE
        }
    }
}

struct TreeElement {
    widget: Widget,
    disposed: bool,
    childs: Vec<usize>,
}

struct Tree {
    elements: Vec<TreeElement>,
    windows: Vec<usize>,
}

impl Tree {
    fn new() -> Tree {
        Tree {
            elements: vec![],
            windows: vec![],
        }
    }

    fn dispose_childs(&mut self, id: usize) {
        for i in 0..self.elements[id].childs.len() {
            let child = self.elements[id].childs[i];
            self.elements[child].disposed = true;
            self.dispose_childs(child);
            self.elements[child].childs.clear();
        }
        self.elements[id].childs.clear();
    }

    fn insert_window(&mut self, new_window: Window) -> usize {
        let window = self
            .elements
            .iter_mut()
            .enumerate()
            .find(|w| w.1.widget.is_window() && w.1.widget.unwrap_window().id == new_window.id);
        if let Some((id, _)) = window {
            self.dispose_childs(id);
            id
        } else {
            self.elements.push(TreeElement {
                widget: Widget::Window(new_window),
                childs: vec![],
                disposed: false,
            });
            let id = self.elements.len() - 1;
            self.windows.push(id);
            id
        }
    }

    fn alloc_in_disposed(&mut self) -> Option<usize> {
        for i in 0..self.elements.len() {
            if self.elements[i].disposed {
                return Some(i);
            }
        }
        None
    }

    fn insert_widget(&mut self, parent: usize, widget: Widget) -> usize {
        let id = self.alloc_in_disposed();

        let id = if let Some(id) = id {
            self.elements[id].widget = widget;
            self.elements[id].disposed = false;
            id
        } else {
            let element = TreeElement {
                widget,
                childs: vec![],
                disposed: false,
            };
            self.elements.push(element);
            self.elements.len() - 1
        };
        self.elements[parent].childs.push(id);

        id
    }
}

enum Layout {
    Vertical,
    Horizontal,
}

struct Cursor {
    x: f32,
    y: f32,
    scroll: Vector2<f32>,
    area: Rect,
}

impl Cursor {
    fn new(area: Rect) -> Cursor {
        Cursor {
            x: consts::MARGIN,
            y: consts::MARGIN,
            scroll: Vector2::new(0., 0.),
            area,
        }
    }

    fn fit(&mut self, size: Vector2<f32>, layout: Layout) -> Point2<f32> {
        let res;

        match layout {
            Layout::Horizontal => {
                if self.x + size.x < self.area.w as f32 - consts::MARGIN * 2. {
                    res = Point2::new(self.x, self.y);
                } else {
                    self.x = consts::MARGIN;
                    self.y += size.y + consts::MARGIN; // TODO: not size.y, but previous row max y, which is currently unknown :(
                    res = Point2::new(self.x, self.y);
                }
                self.x += size.x + consts::MARGIN;
            }
            Layout::Vertical => {
                res = Point2::new(self.x, self.y);
                self.x = consts::MARGIN;
                self.y += size.y + consts::MARGIN
            }
        }
        res
    }
}

struct Input {
    mouse_position: Point2<f32>,
    is_mouse_down: bool,
    click_down: bool,
    click_up: bool,
    mouse_wheel: Vector2<f32>,
}

enum Event {
    ButtonClick(Id),
    Dragging(Id),
    DragDrop(Id, Point2<f32>, Option<Id>),
}
struct Events {
    frame_events: Vec<Event>,
    previous_frame_events: Vec<Event>,
}
impl Events {
    fn new() -> Events {
        Events {
            frame_events: vec![],
            previous_frame_events: vec![],
        }
    }
    fn push(&mut self, event: Event) {
        self.frame_events.push(event)
    }

    fn next_frame(&mut self) {
        self.previous_frame_events.clear();
        self.previous_frame_events.append(&mut self.frame_events);
    }

    fn previous_frame_button_clicked(&self, button_id: Id) -> bool {
        self.previous_frame_events.iter().any(|e| match e {
            Event::ButtonClick(id) => *id == button_id,
            _ => false,
        })
    }

    fn previous_frame_drag(&self, drag_id: Id) -> Drag {
        for event in &self.previous_frame_events {
            if let Event::DragDrop(id, pos, hovered_id) = event {
                if drag_id == *id {
                    return Drag::Dropped(*pos, *hovered_id);
                }
            }
            if let Event::Dragging(id) = event {
                if drag_id == *id {
                    return Drag::Dragging;
                }
            }
        }
        Drag::No
    }
}

#[derive(SmartDefault, Clone)]
struct Scroll {
    dragging_x: bool,
    dragging_y: bool,
    rect: Rect,
    inner_rect: Rect,
    #[default(Vector2::new(0., 0.))]
    initial_scroll: Vector2<f32>,
}

impl Scroll {
    fn scroll_to(&mut self, y: f32) {
        self.rect.y = y
            .max(self.inner_rect.y)
            .min(self.inner_rect.h - self.rect.h + self.inner_rect.y);
    }
}

#[derive(Copy, Clone, Debug)]
enum DragState {
    Clicked(Point2<f32>, Point2<f32>),
    Dragging(Vector2<f32>),
}

#[derive(Copy, Clone, Debug)]
pub enum Drag {
    No,
    Dragging,
    Dropped(Point2<f32>, Option<Id>),
}

pub struct Ui {
    focused: Option<usize>,
    moving: Option<(usize, Vector2<f32>)>,
    dragging: Option<(Id, DragState)>,
    current_element: Option<usize>,
    hovered: Option<Id>,
    scroll_bars: HashMap<Id, Scroll>,
    events: Events,
    tree: Tree,
    input: Input,
}

impl Ui {
    pub fn new() -> Ui {
        Ui {
            focused: None,
            moving: None,
            dragging: None,
            current_element: None,
            hovered: None,
            scroll_bars: HashMap::new(),
            events: Events::new(),
            tree: Tree::new(),
            input: Input {
                mouse_position: Point2::new(0., 0.),
                is_mouse_down: false,
                click_down: false,
                click_up: false,
                mouse_wheel: Vector2::new(0., 0.),
            },
        }
    }
}

impl Ui {
    pub fn window<F: FnOnce(&mut Ui)>(
        &mut self,
        id: Id,
        label: &str,
        position: Point2<f32>,
        size: Vector2<f32>,
        f: F,
    ) {
        let new_window = Window {
            id,
            label: label.to_string(),
            rect: Rect::new(position.x, position.y, size.x, size.y),
        };
        let window_element_id = self.tree.insert_window(new_window);
        self.current_element = Some(window_element_id);

        f(self)
    }

    pub fn button<T: Into<Option<Point2<f32>>>>(&mut self, pos: T, id: Id, label: &str) -> bool {
        self.tree.insert_widget(
            self.current_element.unwrap(),
            Widget::Button {
                pos: pos.into(),
                id: id,
                label: label.to_string(),
            },
        );
        self.events.previous_frame_button_clicked(id)
    }

    pub fn label<T: Into<Option<Point2<f32>>>>(&mut self, pos: T, label: &str) {
        self.tree.insert_widget(
            self.current_element.unwrap(),
            Widget::Label {
                pos: pos.into(),
                label: label.to_string(),
            },
        );
    }

    pub fn mouse_down(&mut self, position: Point2<f32>) {
        self.input.is_mouse_down = true;
        self.input.click_down = true;

        for (n, wid) in self.tree.windows.iter().enumerate() {
            let window = &self.tree.elements[*wid].widget.unwrap_window();

            if window.title_contains(position) {
                self.moving = Some((*wid, position - Point2::new(window.rect.x, window.rect.y)));
            }

            if window.window_contains(position) {
                self.focused = Some(*wid);
                let window = self.tree.windows.remove(n);
                self.tree.windows.insert(0, window);
                return;
            }
        }
        self.focused = None;
    }

    pub fn mouse_up(&mut self, _: Point2<f32>) {
        self.moving = None;
        self.input.is_mouse_down = false;
        self.input.click_up = true;
    }

    pub fn mouse_wheel(&mut self, x: f32, y: f32) {
        self.input.mouse_wheel = Vector2::new(x, y);
    }

    pub fn mouse_move(&mut self, position: Point2<f32>) {
        if let Some((id, orig)) = self.moving.as_ref() {
            let mut window = self
                .tree
                .elements
                .get_mut(*id)
                .unwrap()
                .widget
                .unwrap_window_mut();
            window.rect.x = position.x - orig.x;
            window.rect.y = position.y - orig.y;
        }

        self.input.mouse_position = position;
    }

    pub fn begin_frame(&mut self) {
        self.events.next_frame();
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        for window in self.tree.windows.iter().rev() {
            let mut cursor = Cursor::new(Rect::new(0., 0., 0., 0.));
            draw_element(
                &mut UiContext {
                    elements: &self.tree.elements,
                    ctx: ctx,
                    events: &mut self.events,
                    input: &self.input,
                    dragging: &mut self.dragging,
                    hovered: Some(&mut self.hovered),
                    focused: self.focused.as_ref().map_or(false, |w| w == window),
                    scroll_bars: &mut self.scroll_bars,
                },
                &mut cursor,
                *window,
            );
        }

        if let Some((id, drag)) = self.dragging {
            if let DragState::Clicked(orig, pos) = drag {
                if self.input.mouse_position.distance(orig) > 5. {
                    self.dragging =
                        Some((id, DragState::Dragging(pos - self.input.mouse_position)));
                }
            }
            if let DragState::Dragging(orig) = drag {
                if let Some((n, _)) = self
                    .tree
                    .elements
                    .iter()
                    .enumerate()
                    .find(|(_, e)| e.widget.group_id().map_or(false, |i| i == id))
                {
                    let mut cursor = Cursor::new(Rect::new(
                        self.input.mouse_position.x + orig.x,
                        self.input.mouse_position.y + orig.y,
                        3000.,
                        3000.,
                    ));
                    draw_element(
                        &mut UiContext {
                            elements: &self.tree.elements,
                            ctx: ctx,
                            events: &mut self.events,
                            input: &self.input,
                            dragging: &mut self.dragging,
                            hovered: None,
                            focused: true,
                            scroll_bars: &mut self.scroll_bars,
                        },
                        &mut cursor,
                        n,
                    );
                }
                if self.input.is_mouse_down == false {
                    self.events.push(Event::DragDrop(
                        self.dragging.unwrap().0,
                        self.input.mouse_position,
                        self.hovered,
                    ));
                    self.dragging = None;
                } else {
                    self.events.push(Event::Dragging(self.dragging.unwrap().0));
                }
            }
        }

        self.hovered = None;
        self.input.click_down = false;
        self.input.click_up = false;
        self.input.mouse_wheel = Vector2::new(0., 0.);
    }
}

struct UiContext<'a> {
    elements: &'a [TreeElement],
    ctx: &'a mut Context,
    events: &'a mut Events,
    input: &'a Input,
    focused: bool,
    dragging: &'a mut Option<(Id, DragState)>,
    scroll_bars: &'a mut HashMap<Id, Scroll>,
    hovered: Option<&'a mut Option<Id>>,
}

fn draw_element(context: &mut UiContext, cursor: &mut Cursor, id: usize) -> Rect {
    let element = &context.elements[id];
    let widget = &element.widget;
    let orig = Vector2::new(cursor.area.x as f32, cursor.area.y as f32) + cursor.scroll;

    let rect = match widget {
        Widget::Window(window) => {
            let inside_rect = Rect::new(
                window.rect.x + consts::MARGIN,
                window.rect.y + consts::TITLE_HEIGHT + consts::MARGIN,
                window.rect.w - consts::MARGIN,
                window.rect.h - consts::TITLE_HEIGHT - consts::MARGIN,
            );

            draw_window_frame(context.ctx, context.focused, window);
            draw_scroll_area(context, window.id, inside_rect, &element.childs);

            window.rect
        }
        Widget::Label { pos, label } => {
            let size = context.ctx.canvas_context().measure_label(label, None);
            let pos = pos.unwrap_or(cursor.fit(size, Layout::Vertical)) + orig;

            context.ctx.canvas_context().draw_label(
                label,
                pos,
                None,
                None,
                Some(consts::text(context.focused)),
            );

            Rect::new(pos.x, pos.y, size.x as f32, size.y as f32)
        }
        Widget::Button { pos, label, id, .. } => {
            let size = context.ctx.canvas_context().measure_label(label, None)
                + Vector2::new(
                    consts::MARGIN_BUTTON as f32 * 2.,
                    consts::MARGIN_BUTTON as f32 * 2.,
                );
            let pos = pos.unwrap_or(cursor.fit(size, Layout::Vertical)) + orig;
            let rect = Rect::new(pos.x, pos.y, size.x as f32, size.y as f32);
            let hovered = rect.contains(context.input.mouse_position);

            context.ctx.canvas_context().draw_rect(
                rect,
                &[RectAttr::Fill(consts::button_background(
                    context.focused,
                    hovered,
                    hovered && context.input.is_mouse_down,
                ))],
            );
            context.ctx.canvas_context().draw_label(
                label,
                pos + Vector2::new(consts::MARGIN_BUTTON, consts::MARGIN_BUTTON),
                None,
                None,
                Some(consts::text(context.focused)),
            );

            if context.focused && hovered && context.input.click_up {
                context.events.push(Event::ButtonClick(id.clone()));
            }
            rect
        }
        Widget::Group(widgets::Group {
            size,
            id,
            draggable,
            hoverable,
            highlight,
        }) => {
            let pos: Point2<f32> = cursor.fit(*size, Layout::Horizontal);
            let rect = Rect::new(pos.x + orig.x, pos.y + orig.y, size.x as f32, size.y as f32);
            let mut hovered = false;

            if *draggable {
                if element.childs.len() != 0
                    && context.dragging.is_none()
                    && context.input.is_mouse_down
                    && rect.contains(context.input.mouse_position)
                {
                    *context.dragging = Some((
                        *id,
                        DragState::Clicked(
                            context.input.mouse_position,
                            Point2::new(rect.x, rect.y),
                        ),
                    ));
                }
            }

            if *draggable || *hoverable {
                if rect.contains(context.input.mouse_position) {
                    if let Some(ref mut hover) = context.hovered {
                        **hover = Some(*id);
                        hovered = true;
                    }
                }
            }
            if rect.overlaps(&cursor.area) {
                draw_group_frame(
                    context.ctx,
                    context.focused,
                    context.hovered.is_some() && hovered,
                    *highlight,
                    rect,
                );
                draw_scroll_area(context, *id, rect, &element.childs);
            }

            rect
        }
    };

    Rect::new(rect.x - orig.x, rect.y - orig.y, rect.w, rect.h)
}

fn extend_rect(left: Rect, right: Rect) -> Rect {
    let mut rect = left;

    if right.x < left.x {
        rect.x = right.x;
        rect.w += left.x - right.x;
    }
    if right.y < left.y {
        rect.y = right.y;
        rect.h += left.y - right.y;
    }

    if left.x + left.w < right.x + right.w {
        rect.w += right.x + right.w - (left.x + left.w);
    }
    if left.y + left.h < right.y + right.h {
        rect.h += right.y + right.h - (left.y + left.h);
    }

    rect
}

fn draw_scroll_area(context: &mut UiContext, id: Id, rect: Rect, elements: &[usize]) {
    let mut cursor = Cursor::new(rect);
    let mut inner_rect = Rect::new(0., 0., rect.w, rect.h);
    {
        let scroll = context.scroll_bars.entry(id).or_insert(Scroll {
            rect: Rect::new(0., 0., rect.w, rect.h),
            ..Default::default()
        });
        cursor.scroll = Vector2::new(-scroll.rect.x, -scroll.rect.y);
    }

    {
        let canvas2d = &context.ctx.canvas_context().canvas;
        canvas2d.save();
        canvas2d.rect(rect.x as f64, rect.y as f64, rect.w as f64, rect.h as f64);
        canvas2d.clip(stdweb::web::FillRule::NonZero);
    }

    for child in elements {
        let element_rect = draw_element(context, &mut cursor, *child);
        inner_rect = extend_rect(inner_rect, element_rect);
    }

    let mut scroll = context.scroll_bars.get_mut(&id).unwrap();
    scroll.inner_rect = inner_rect;

    if inner_rect.h > rect.h {
        draw_vertical_scroll_bar(
            context,
            rect,
            Rect::new(
                rect.x + rect.w - consts::SCROLL_WIDTH,
                rect.y,
                consts::SCROLL_WIDTH,
                rect.h,
            ),
            id,
        );
    }

    context.ctx.canvas_context().canvas.restore();
}

fn draw_vertical_scroll_bar(context: &mut UiContext, area: Rect, rect: Rect, id: Id) {
    let mut scroll = context.scroll_bars.get_mut(&id).unwrap();
    let size = scroll.rect.h / scroll.inner_rect.h * rect.h;
    let pos = (scroll.rect.y - scroll.inner_rect.y) / scroll.inner_rect.h * rect.h;

    context.ctx.canvas_context().draw_line(
        Point2::new(rect.x, rect.y),
        Point2::new(rect.x, rect.y + rect.h),
        consts::window_border(context.focused),
    );

    let mut clicked = false;
    let mut hovered = false;
    let bar = Rect::new(rect.x + 1., rect.y + pos, rect.w - 1., size);
    let k = scroll.inner_rect.h / scroll.rect.h;
    if bar.contains(context.input.mouse_position) {
        hovered = true;
    }
    if hovered && context.input.click_down {
        scroll.dragging_y = true;
        scroll.initial_scroll.y = scroll.rect.y - context.input.mouse_position.y * k;
    }
    if context.input.is_mouse_down == false {
        scroll.dragging_y = false;
    }
    if scroll.dragging_y {
        clicked = true;
        scroll.scroll_to(context.input.mouse_position.y * k + scroll.initial_scroll.y);
    }

    if context.focused
        && area.contains(context.input.mouse_position)
        && context.input.mouse_wheel.y != 0.
    {
        scroll
            .scroll_to(scroll.rect.y + context.input.mouse_wheel.y * k * consts::SCROLL_MULTIPLIER);
    }

    context.ctx.canvas_context().draw_rect(
        bar,
        &[RectAttr::Fill(consts::scroll_bar_handle(
            context.focused,
            hovered,
            clicked,
        ))],
    );
}

fn draw_group_frame(ctx: &mut Context, focused: bool, hovered: bool, highlight: bool, rect: Rect) {
    ctx.canvas_context().draw_rect(
        rect,
        &[RectAttr::Stroke(consts::drag_border(
            focused, hovered, highlight,
        ))],
    );
}

#[allow(dead_code)]
enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

#[allow(dead_code)]
fn draw_arrow(
    ctx: &mut Context,
    pos: Point2<f32>,
    size: Vector2<f32>,
    dir: ArrowDirection,
    focused: bool,
) {
    let p1;
    let p2;
    let p3;

    match dir {
        ArrowDirection::Up => {
            p1 = Point2::new(pos.x, pos.y + size.y);
            p2 = Point2::new(pos.x + size.x / 2., pos.y);
            p3 = Point2::new(pos.x + size.x, pos.y + size.y);
        }
        ArrowDirection::Down => {
            p1 = Point2::new(pos.x, pos.y);
            p2 = Point2::new(pos.x + size.x / 2., pos.y + size.y);
            p3 = Point2::new(pos.x + size.x, pos.y);
        }
        ArrowDirection::Left => {
            p1 = Point2::new(pos.x + size.x, pos.y);
            p2 = Point2::new(pos.x, pos.y + size.y / 2.);
            p3 = Point2::new(pos.x + size.x, pos.y + size.y);
        }
        ArrowDirection::Right => {
            p1 = Point2::new(pos.x, pos.y);
            p2 = Point2::new(pos.x + size.x, pos.y + size.y / 2.);
            p3 = Point2::new(pos.x, pos.y + size.y);
        }
    }

    ctx.canvas_context()
        .draw_line(p1, p2, consts::window_border(focused));
    ctx.canvas_context()
        .draw_line(p2, p3, consts::window_border(focused));
}

fn draw_window_frame(ctx: &mut Context, focused: bool, window: &Window) {
    let Window { label, rect, .. } = window;

    ctx.canvas_context().draw_rect(
        *rect,
        &[
            RectAttr::Stroke(consts::window_border(focused)),
            RectAttr::Fill(consts::background(focused)),
        ],
    );
    ctx.canvas_context().draw_label(
        &label,
        Point2::new(rect.x + consts::MARGIN, rect.y + consts::MARGIN),
        None,
        None,
        Some(consts::title(focused)),
    );
    ctx.canvas_context().draw_line(
        Point2::new(rect.x, rect.y + consts::TITLE_HEIGHT),
        Point2::new(rect.x + rect.w, rect.y + consts::TITLE_HEIGHT),
        consts::window_border(focused),
    );
}
