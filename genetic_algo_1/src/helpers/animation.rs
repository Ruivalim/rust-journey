use bevy::prelude::*;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct AnimationState {
    pub name: String,
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[allow(dead_code)]
impl AnimationState {
    pub fn new(
        name: String,
        first_sprite_index: usize,
        last_sprite_index: usize,
    ) -> AnimationState {
        AnimationState {
            name,
            first_sprite_index,
            last_sprite_index,
            flip_x: false,
            flip_y: false,
        }
    }

    pub fn flip_x(&mut self) -> Self {
        self.flip_x = !self.flip_x;

        self.clone()
    }

    pub fn flip_y(&mut self) -> Self {
        self.flip_y = !self.flip_y;

        self.clone()
    }
}

#[allow(dead_code)]
#[derive(Component)]
pub struct AnimationConfig {
    pub current_animation_first_sprite_index: usize,
    pub current_animation_last_sprite_index: usize,
    pub current_animation_flip_x: bool,
    pub current_animation_flip_y: bool,
    pub current_animation_name: String,
    animations: Vec<AnimationState>,
    pub fps: u8,
    pub frame_timer: Timer,
    update_animation: bool,
}

impl AnimationConfig {
    pub fn new(animations: Vec<AnimationState>, fps: u8) -> Self {
        Self {
            current_animation_first_sprite_index: animations[0].first_sprite_index,
            current_animation_last_sprite_index: animations[0].last_sprite_index,
            current_animation_flip_x: animations[0].flip_x,
            current_animation_flip_y: animations[0].flip_y,
            current_animation_name: animations[0].name.clone(),
            animations,
            fps,
            frame_timer: Self::timer_from_fps(fps),
            update_animation: false,
        }
    }

    pub fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(
            Duration::from_secs_f32(1.0 / (fps as f32)),
            TimerMode::Repeating,
        )
    }

    pub fn update_animation(&mut self, to: String) {
        if self.current_animation_name == to {
            return;
        }
        let animations = self.animations.iter();
        for animation in animations {
            if animation.name == to {
                self.current_animation_first_sprite_index = animation.first_sprite_index;
                self.current_animation_last_sprite_index = animation.last_sprite_index;
                self.current_animation_flip_x = animation.flip_x;
                self.current_animation_flip_y = animation.flip_y;
                self.current_animation_name = to.clone();
                self.update_animation = true;
            }
        }
    }

    pub fn execute_animations(&mut self, time: Duration, sprite: &mut Sprite) {
        let atlas = sprite.texture_atlas.as_mut().unwrap();
        if self.update_animation == true {
            atlas.index = self.current_animation_first_sprite_index;
            sprite.flip_x = self.current_animation_flip_x;
            sprite.flip_y = self.current_animation_flip_y;
            self.update_animation = false;
        }
        self.frame_timer.tick(time);

        if self.frame_timer.just_finished() {
            if atlas.index == self.current_animation_last_sprite_index {
                atlas.index = self.current_animation_first_sprite_index;
            } else {
                atlas.index += 1;
                self.frame_timer = AnimationConfig::timer_from_fps(self.fps);
            }
        }
    }
}

pub fn execute_animations_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut Sprite), With<AnimationConfig>>,
) {
    for (animator, sprite) in query.iter_mut() {
        let sprite = sprite.into_inner();
        let animator = animator.into_inner();
        animator.execute_animations(time.delta(), sprite);
    }
}
