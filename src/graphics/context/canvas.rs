use stdweb::{
    traits::*,
    web::{CanvasRenderingContext2d, FillRule, TextAlign, TextBaseline},
};

use crate::graphics::types::Rect;

use crate::goodies::matrix_transform_2d::Transform2d;
use cgmath::{EuclideanSpace, InnerSpace, Matrix3, Point2, Vector2};

pub struct CanvasContext {
    pub canvas: CanvasRenderingContext2d,    
}

impl CanvasContext {
    pub fn new(canvas: CanvasRenderingContext2d) -> Self {
        CanvasContext {
            canvas,        
        }
    }
}

impl CanvasContext {
    pub(crate) fn set_transform_with_matrix(&self, matrix: &Matrix3<f32>) {
        self.canvas.set_transform(
            matrix.x.x as f64,
            matrix.x.y as f64,
            matrix.y.x as f64,
            matrix.y.y as f64,
            matrix.z.x as f64,
            matrix.z.y as f64,
        );
    }

    pub(crate) fn transform_with_matrix(&self, matrix: &Matrix3<f32>) {
        self.canvas.transform(
            matrix.x.x as f64,
            matrix.x.y as f64,
            matrix.y.x as f64,
            matrix.y.y as f64,
            matrix.z.x as f64,
            matrix.z.y as f64,
        );
    }

    pub fn set_transform(&self, transform: &Matrix3<f32>) {
        self.set_transform_with_matrix(transform);
    }

    pub fn push_transform(&self, transform: &Matrix3<f32>) {
        self.canvas.save();
        self.transform_with_matrix(transform);
    }

    pub fn pop_transform(&self) {
        self.canvas.restore();
    }

    pub fn size(&self) -> (f64, f64) {
        let width = self.canvas.get_canvas().offset_width();
        let height = self.canvas.get_canvas().offset_height();

        (width as f64, height as f64)
    }

    pub fn set_screen_coordinates(&mut self, rect: Rect) {
        use crate::matrix_transform_2d::Transform2d;
        use cgmath::Matrix3;
        
        let (width, height) = self.size();
        let translate = Matrix3::from_translation(Vector2::new(-rect.x, -rect.y));
        let scale = Matrix3::from_nonuniform_scale(width as f32 / rect.w, height as f32 / rect.h);
        let transform = scale * translate;

        self.set_transform(&transform);
    }

    pub fn clear(&self) {
        let size = self.size();

        self.canvas.save();
        self.set_transform(&cgmath::One::one());
        self.canvas
            .clear_rect(0.0f64, 0.0f64, size.0 as f64, size.1 as f64);
        self.canvas.restore();
    }

    pub fn draw_arc(&self, position: Point2<f64>, radius: f64, attrs: &[ArcAttr]) {
        self.canvas.save();

        let mut angle = std::f64::consts::PI * 2.0;
        let mut start_angle = 0.;
        let mut sector = false;
        let mut forward = false;
        for attr in attrs.iter() {
            match attr {
                ArcAttr::Stroke(color) => {
                    self.canvas.set_stroke_style_color(color);
                }
                ArcAttr::Fill(color) => {
                    self.canvas.set_fill_style_color(color);
                }
                ArcAttr::Dash(dash) => self.canvas.set_line_dash(dash.clone()),
                ArcAttr::Angle(a) => angle = *a,
                ArcAttr::Forward(dir) => {
                    start_angle = Vector2::new(dir.x, -dir.y).angle(Vector2::new(-1., 0.)).0;
                    forward = true;
                }
                ArcAttr::Sector => sector = true,
            }
        }
        if forward == false {
            start_angle = angle / 2.;
        }

        self.canvas.begin_path();
        if sector {
            self.canvas.move_to(position.x, position.y);
        }
        self.canvas.arc(
            position.x,
            position.y,
            radius,
            start_angle - angle / 2.,
            start_angle + angle / 2.,
            false,
        );
        if sector {
            self.canvas.move_to(position.x, position.y);
        }
        for attr in attrs.iter() {
            match attr {
                ArcAttr::Stroke(_) => self.canvas.stroke(),
                ArcAttr::Fill(_) => {
                    self.canvas.fill(FillRule::NonZero);
                }
                _ => {}
            }
        }
        self.canvas.restore();
    }

    pub fn draw_line(&self, from: Point2<f64>, to: Point2<f64>, color: &str) {
        self.canvas.save();
        self.canvas.set_stroke_style_color(color);
        self.canvas.begin_path();
        self.canvas.move_to(from.x, from.y);
        self.canvas.line_to(to.x, to.y);
        self.canvas.stroke();
        self.canvas.restore();
    }

    pub fn draw_label(
        &self,
        label: &str,
        pos: Point2<f64>,
        scale: Option<Vector2<f64>>,
        font: Option<&str>,
        color: Option<&str>,
    ) {
        self.canvas.save();
        if let Some(scale) = scale {
            let scale = Matrix3::from_nonuniform_scale(scale.x as f32, scale.y as f32);
            let pos = Matrix3::from_translation(pos.cast::<f32>().unwrap().to_vec());
            let transform = pos * scale;

            self.push_transform(&transform);
        }
        if let Some(color) = color {
            self.canvas.set_fill_style_color(color);
        }
        if font.is_some() {
            self.canvas.set_font(font.unwrap());
        }

        self.canvas.set_text_align(TextAlign::Left);
        self.canvas.set_text_baseline(TextBaseline::Hanging);

        if scale.is_none() {
            self.canvas.fill_text(label, pos.x, pos.y, None);
        } else {
            self.canvas.fill_text(label, 0., 0., None);
        }

        if scale.is_some() {
            self.pop_transform();
        }
        self.canvas.restore();
    }

    pub fn measure_label(
        &self,
        label: &str,
        _pos: Point2<f64>,
        font: Option<&str>,
        color: Option<&str>,
    ) -> Vector2<f64> {
        self.canvas.save();
        if let Some(color) = color {
            self.canvas.set_fill_style_color(color);
        }
        if font.is_some() {
            self.canvas.set_font(font.unwrap());
        }
        self.canvas.set_text_align(TextAlign::Left);
        self.canvas.set_text_baseline(TextBaseline::Hanging);
        let measures = self.canvas.measure_text(label).unwrap();
        self.canvas.restore();

        Vector2::new(measures.get_width(), 10.)
    }
}

pub enum ArcAttr {
    Stroke(&'static str),
    Fill(&'static str),
    Dash(Vec<f64>),
    Angle(f64),
    Forward(Vector2<f64>),
    Sector,
}
