use nalgebra::{Vector3, Matrix4, UnitQuaternion, Quaternion, Point3};

#[derive(Debug, Clone)]
pub enum CameraType {
    FirstPerson,
    ThirdPerson,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    pub camera_type: CameraType,
    pub offset: Vector3<f32>, // Offset for third person view
    pub rotation: UnitQuaternion<f32>,
}

impl Camera {
    pub fn new(
        position: Vector3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            target,
            up,
            fov,
            aspect_ratio,
            near,
            far,
            camera_type: CameraType::ThirdPerson,
            offset: Vector3::new(0.0, 2.0, -5.0), // Default offset for third person
            rotation: UnitQuaternion::identity(),
        }
    }

    pub fn new_with_rotation(
        position: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let forward = rotation.transform_vector(&Vector3::new(0.0, 0.0, 1.0));
        let target = position + forward;

        Self {
            position,
            target,
            up: Vector3::new(0.0, 1.0, 0.0),
            fov,
            aspect_ratio,
            near,
            far,
            camera_type: CameraType::FirstPerson,
            offset: Vector3::new(0.0, 0.0, 0.0),
            rotation,
        }
    }

    pub fn switch_to_first_person(&mut self, truck_position: Vector3<f32>, truck_rotation: UnitQuaternion<f32>) {
        self.camera_type = CameraType::FirstPerson;
        // Position camera at truck's position with slight height adjustment
        self.position = truck_position + Vector3::new(0.0, 1.5, 0.0);
        // Set camera to look in the same direction as the truck
        self.rotation = truck_rotation;
        let forward = self.rotation.transform_vector(&Vector3::new(0.0, 0.0, 1.0));
        self.target = self.position + forward;
    }

    pub fn switch_to_third_person(&mut self, truck_position: Vector3<f32>, truck_rotation: UnitQuaternion<f32>) {
        self.camera_type = CameraType::ThirdPerson;
        // Position camera behind and above the truck
        let backward = truck_rotation.transform_vector(&Vector3::new(0.0, 0.0, -1.0));
        let offset = Vector3::new(0.0, 2.0, -5.0); // Standard offset
        self.position = truck_position + backward * offset.z + Vector3::new(0.0, offset.y, 0.0);
        self.target = truck_position;
        self.rotation = truck_rotation;
    }

    pub fn update_for_truck(&mut self, truck_position: Vector3<f32>, truck_rotation: UnitQuaternion<f32>) {
        match self.camera_type {
            CameraType::FirstPerson => {
                self.position = truck_position + Vector3::new(0.0, 1.5, 0.0);
                self.rotation = truck_rotation;
                let forward = self.rotation.transform_vector(&Vector3::new(0.0, 0.0, 1.0));
                self.target = self.position + forward;
            }
            CameraType::ThirdPerson => {
                let backward = truck_rotation.transform_vector(&Vector3::new(0.0, 0.0, -1.0));
                let offset = Vector3::new(0.0, 2.0, -5.0);
                self.position = truck_position + backward * offset.z + Vector3::new(0.0, offset.y, 0.0);
                self.target = truck_position;
            }
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let pos = Point3::from(self.position);
        let tgt = Point3::from(self.target);
        Matrix4::look_at_rh(&pos, &tgt, &self.up)
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(self.aspect_ratio, self.fov, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        (self.target - self.position).normalize()
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }
}