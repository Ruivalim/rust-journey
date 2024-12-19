use ggez::{
    glam::Vec2,
    graphics::{Color, DrawMode, DrawParam, Mesh, Rect},
};

use nalgebra::Point2;
use rapier2d::prelude::*;

pub struct Physics {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    gravity: Vector<f32>,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    query_pipeline: QueryPipeline,
}

pub struct Ball {
    pub mesh: Mesh,
    pub body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub position_initial: Vec2,
    pub size: Vec2,
}

pub struct Cuboid {
    pub mesh: Mesh,
    pub body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub position_initial: Vec2,
    pub size: Vec2,
}

impl Physics {
    pub fn new(gravity: Vector<f32>) -> Self {
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let ccd_solver = CCDSolver::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let query_pipeline = QueryPipeline::new();

        Self {
            rigid_body_set,
            collider_set,
            integration_parameters,
            physics_pipeline,
            gravity,
            island_manager,
            broad_phase,
            narrow_phase,
            ccd_solver,
            physics_hooks: (),
            event_handler: (),
            impulse_joint_set,
            multibody_joint_set,
            query_pipeline,
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &mut self.physics_hooks,
            &mut self.event_handler,
        );
    }

    pub fn new_ball(
        &mut self,
        ctx: &ggez::Context,
        identifier: u128,
        ball_init_x: f32,
        ball_init_y: f32,
        ball_radius: f32,
        color: Color,
        restitution: f32,
        fixed: bool,
    ) -> Ball {
        let mut ball_body = if fixed {
            RigidBodyBuilder::fixed()
        } else {
            RigidBodyBuilder::dynamic()
        }
        .position(Isometry::translation(ball_init_x, ball_init_y))
        .build();

        ball_body.user_data = identifier;

        let ball_handle: RigidBodyHandle = self.rigid_body_set.insert(ball_body);

        let mut ball_collider = ColliderBuilder::ball(ball_radius)
            .restitution(restitution)
            .build();

        ball_collider.user_data = identifier;

        let ball_collider_handle = self.collider_set.insert_with_parent(
            ball_collider,
            ball_handle,
            &mut self.rigid_body_set,
        );

        let mesh = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            ball_radius,
            0.1,
            color,
        )
        .unwrap();

        return Ball {
            mesh,
            body_handle: ball_handle,
            collider_handle: ball_collider_handle,
            position_initial: Vec2::new(ball_init_x, ball_init_y),
            size: Vec2::new(ball_radius * 2.0, ball_radius * 2.0),
        };
    }

    pub fn new_cuboid(
        &mut self,
        ctx: &ggez::Context,
        identifier: u128,
        cuboid_width: f32,
        cuboid_height: f32,
        cuboid_x: f32,
        cuboid_y: f32,
        color: Color,
        restitution: f32,
        fixed: bool,
    ) -> Cuboid {
        let mut cuboid_body = if fixed {
            RigidBodyBuilder::fixed()
        } else {
            RigidBodyBuilder::dynamic()
        }
        .position(Isometry::translation(
            cuboid_x + cuboid_width / 2.0,
            cuboid_y + cuboid_height / 2.0,
        ))
        .build();

        cuboid_body.user_data = identifier;
        let cuboid_handle: RigidBodyHandle = self.rigid_body_set.insert(cuboid_body);

        let mut cuboid_collider = ColliderBuilder::cuboid(cuboid_width / 2.0, cuboid_height / 2.0)
            .restitution(restitution)
            .build();

        cuboid_collider.user_data = identifier;

        let cuboid_collider_handle = self.collider_set.insert_with_parent(
            cuboid_collider,
            cuboid_handle,
            &mut self.rigid_body_set,
        );

        let mesh = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            ggez::graphics::Rect::new(
                -cuboid_width / 2.0,
                -cuboid_height / 2.0,
                cuboid_width,
                cuboid_height,
            ),
            color,
        )
        .unwrap();

        return Cuboid {
            mesh,
            body_handle: cuboid_handle,
            collider_handle: cuboid_collider_handle,
            position_initial: Vec2::new(cuboid_x, cuboid_y),
            size: Vec2::new(cuboid_width, cuboid_height),
        };
    }

    pub fn render_gizmos(&self, ctx: &ggez::Context) -> Vec<(Mesh, DrawParam)> {
        let mut gizmos = Vec::new();

        for collider in self.collider_set.iter() {
            let collider = collider.1;

            let position = collider.position();
            let rotation = position.rotation.angle();
            let translation = position.translation;
            let shape = collider.shape();

            if let Some(ball) = shape.as_ball() {
                gizmos.push((
                    Mesh::new_circle(
                        ctx,
                        DrawMode::stroke(1.0),
                        Vec2::new(translation.x, translation.y),
                        ball.radius,
                        0.1,
                        Color::RED,
                    )
                    .expect("Failed to create ball gizmo"),
                    DrawParam::default(),
                ));
            } else if let Some(cuboid) = shape.as_cuboid() {
                let half_extents = cuboid.half_extents;
                let position = collider.position();
                let rotation = position.rotation.angle();

                let w = 2.0 * half_extents.x;
                let h = 2.0 * half_extents.y;

                let mesh = Mesh::new_rectangle(
                    ctx,
                    DrawMode::stroke(1.0),
                    Rect::new(-half_extents.x, -half_extents.y, w, h),
                    Color::RED,
                )
                .expect("Failed to create cuboid gizmo");

                gizmos.push((
                    mesh,
                    DrawParam::default()
                        .dest(Vec2::new(translation.x, translation.y))
                        .rotation(rotation),
                ));
            }
        }

        gizmos
    }

    pub fn apply_impulse(&mut self, body_handle: RigidBodyHandle, force: Vector<f32>) {
        if let Some(body) = self.rigid_body_set.get_mut(body_handle) {
            body.apply_impulse(force, true);
        }
    }

    pub fn query_point(&self, point: Vector<f32>) -> Vec<ColliderHandle> {
        let mut colliders = Vec::new();
        let query_filter = QueryFilter::default();

        self.query_pipeline.intersections_with_point(
            &self.rigid_body_set,
            &self.collider_set,
            &Point2::new(point.x, point.y),
            query_filter,
            &mut |collider_handle| {
                colliders.push(collider_handle);
                true
            },
        );

        colliders
    }

    pub fn apply_impulse_to_coordenates(
        &mut self,
        x: f32,
        y: f32,
        collider_handle: ColliderHandle,
    ) {
        let body_handle = self.collider_set[collider_handle].parent().unwrap();
        let body = self.rigid_body_set.get_mut(body_handle).unwrap();

        let current_pos = body.position().translation.vector;
        let target_pos = Vector::new(x, y);
        let delta_pos = target_pos - current_pos;

        let acceleration = delta_pos * 0.1;

        let mass = body.mass();

        let force = mass * acceleration;

        body.apply_impulse(force, true);
    }

    pub fn move_to(&mut self, x: f32, y: f32, collider_handle: ColliderHandle) {
        let body_handle = self.collider_set[collider_handle].parent().unwrap();
        let body = self.rigid_body_set.get_mut(body_handle).unwrap();

        body.set_position(Isometry::translation(x, y), true);
    }
}

pub fn string_to_u128(input: &str) -> u128 {
    let mut bytes = [0u8; 16];
    let input_bytes = input.as_bytes();
    if input_bytes.len() > 16 {
        panic!("String is too long to fit in a u128");
    }

    bytes[..input_bytes.len()].copy_from_slice(input_bytes);

    u128::from_be_bytes(bytes)
}

pub fn u128_to_string(value: u128) -> String {
    let bytes = value.to_be_bytes();

    let mut bytes = bytes.to_vec();
    bytes.retain(|&x| x != 0);

    String::from_utf8(bytes.to_vec()).expect("Failed to convert bytes back to a valid UTF-8 string")
}
