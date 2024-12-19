use ggez::{
    glam::Vec2,
    graphics::{Color, DrawMode, Mesh, Rect},
    GameError,
};

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
        ball_init_x: f32,
        ball_init_y: f32,
        ball_radius: f32,
        color: Color,
        restitution: f32,
        fixed: bool,
    ) -> (Mesh, RigidBodyHandle) {
        let ball_body = if fixed {
            RigidBodyBuilder::fixed()
        } else {
            RigidBodyBuilder::dynamic()
        }
        .position(Isometry::translation(ball_init_x, ball_init_y))
        .build();
        let ball_handle: RigidBodyHandle = self.rigid_body_set.insert(ball_body);

        let ball_collider = ColliderBuilder::ball(ball_radius)
            .restitution(restitution)
            .build();
        self.collider_set
            .insert_with_parent(ball_collider, ball_handle, &mut self.rigid_body_set);

        let mesh = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            ball_radius,
            0.1,
            color,
        )
        .unwrap();

        return (mesh, ball_handle);
    }

    pub fn new_cuboid(
        &mut self,
        ctx: &ggez::Context,
        cuboid_width: f32,
        cuboid_height: f32,
        cuboid_x: f32,
        cuboid_y: f32,
        color: Color,
        restitution: f32,
        fixed: bool,
    ) -> (Mesh, RigidBodyHandle, Vec2) {
        let cuboid_body = if fixed {
            RigidBodyBuilder::fixed()
        } else {
            RigidBodyBuilder::dynamic()
        }
        .position(Isometry::translation(
            cuboid_x + cuboid_width / 2.0,
            cuboid_y + cuboid_height / 2.0,
        ))
        .build();

        let cuboid_handle: RigidBodyHandle = self.rigid_body_set.insert(cuboid_body);

        let cuboid_collider = ColliderBuilder::cuboid(cuboid_width / 2.0, cuboid_height / 2.0)
            .restitution(restitution)
            .build();
        self.collider_set.insert_with_parent(
            cuboid_collider,
            cuboid_handle,
            &mut self.rigid_body_set,
        );

        let mesh = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0.0, 0.0, cuboid_width, cuboid_height),
            color,
        )
        .unwrap();

        return (mesh, cuboid_handle, Vec2::new(cuboid_x, cuboid_y));
    }

    pub fn render_gizmos(&self, ctx: &ggez::Context) -> Vec<Mesh> {
        let mut gizmos = Vec::new();

        for collider in self.collider_set.iter() {
            let collider = collider.1;

            let position = collider.position();
            let translation = position.translation;
            let shape = collider.shape();

            if let Some(ball) = shape.as_ball() {
                gizmos.push(
                    Mesh::new_circle(
                        ctx,
                        DrawMode::stroke(1.0),
                        Vec2::new(translation.x, translation.y),
                        ball.radius,
                        0.1,
                        Color::RED,
                    )
                    .expect("Failed to create ball gizmo"),
                );
            } else if let Some(cuboid) = shape.as_cuboid() {
                let half_extents = cuboid.half_extents;
                gizmos.push(
                    Mesh::new_rectangle(
                        ctx,
                        DrawMode::stroke(1.0),
                        Rect::new(
                            translation.x - half_extents.x,
                            translation.y - half_extents.y,
                            half_extents.x * 2.0,
                            half_extents.y * 2.0,
                        ),
                        Color::RED,
                    )
                    .expect("Failed to create cuboid gizmo"),
                )
            }
        }

        gizmos
    }
}
