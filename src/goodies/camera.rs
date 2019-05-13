use cgmath::{Matrix3, Matrix4, Point2, Rad, SquareMatrix, Transform, Vector2};

use crate::goodies::matrix_transform_2d::*;

pub struct Camera {
    pub position: Point2<f32>,
    pub visible_field_width: f32,
    pub rotation: f32,

    pub screen_width: f32,
    pub screen_height: f32,

    canvas_matrix: Matrix3<f32>,
    gl_matrix: Matrix4<f32>,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            position: Point2::new(0., 0.),
            visible_field_width: 500.,
            rotation: 0.,
            screen_width: 100.,
            screen_height: 100.,
            canvas_matrix: Matrix3::identity(),
            gl_matrix: Matrix4::identity(),
        }
    }
}

impl Camera {
    pub fn screen_to_world_point(&self, point: Point2<f32>) -> Point2<f32> {
        self.canvas_matrix.invert().unwrap().transform_point(point)
    }

    pub fn screen_to_world_vector(&self, vector: Vector2<f32>) -> Vector2<f32> {
        let matrix = self.canvas_matrix.invert().unwrap();

        Transform::<Point2<f32>>::transform_vector(&matrix, vector)
    }

    pub fn set_visible_field(&mut self, field: f32) {
        self.visible_field_width = field;
        self.update_matrix();
    }

    pub fn update_screen_size(&mut self, w: f32, h: f32) {
        self.screen_width = w;
        self.screen_height = h;

        self.update_matrix();
    }

    pub fn world_to_screen_point(&self, point: Point2<f32>) -> Point2<f32> {
        self.canvas_matrix.transform_point(point)
    }

    pub fn world_to_screen_vector(&self, vector: Vector2<f32>) -> Vector2<f32> {
        Transform::<Point2<f32>>::transform_vector(&self.canvas_matrix, vector)
    }

    pub fn canvas_matrix(&self) -> Matrix3<f32> {
        self.canvas_matrix
    }

    pub fn gl_matrix(&self) -> Matrix4<f32> {
        self.gl_matrix
    }

    pub fn set_position(&mut self, position: Point2<f32>) {
        self.position = position;
        self.update_matrix();
    }

    fn update_matrix(&mut self) {
        let screen_center = Vector2::new(self.screen_width / 2., self.screen_height / 2.);
        let camera_center = self.position;

        let translate0 = Matrix3::from_translation(screen_center);
        let scale = Matrix3::from_nonuniform_scale(
            self.screen_width / self.visible_field_width,
            self.screen_width / self.visible_field_width,
        );
        let translate1 =
            Matrix3::from_translation(Vector2::new(-camera_center.x, -camera_center.y));

        let rotation = Matrix3::from_angle_z(Rad(self.rotation));

        self.canvas_matrix = translate0 * rotation * scale * translate1;

        self.gl_matrix = cgmath::ortho(
            camera_center.x - self.visible_field_width / 2.,
            camera_center.x + self.visible_field_width / 2.,
            camera_center.y
                + self.screen_height / self.screen_width * self.visible_field_width / 2.,
            camera_center.y
                - self.screen_height / self.screen_width * self.visible_field_width / 2.,
            -1.0,
            1.0,
        );
    }
}
