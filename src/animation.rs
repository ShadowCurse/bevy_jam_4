use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_animations);
    }
}

#[derive(Component)]
pub struct Animation {
    pub animate_forward: bool,
    pub animate_backward: bool,
    pub animation_speed: f32,
    pub progress: f32,
    pub initial_transform: Transform,
    pub target_transform: Transform,
}

fn run_animations(
    time: Res<Time>,
    mut commands: Commands,
    mut animations: Query<(Entity, &mut Animation, &mut Transform)>,
) {
    for (hud, mut animation, mut transform) in animations.iter_mut() {
        animation.progress += time.delta_seconds() * animation.animation_speed;

        if animation.animate_forward {
            transform.translation = animation
                .initial_transform
                .translation
                .lerp(animation.target_transform.translation, animation.progress);
            transform.rotation = animation
                .initial_transform
                .rotation
                .lerp(animation.target_transform.rotation, animation.progress);

            if 1.0 <= animation.progress {
                if animation.animate_backward {
                    animation.progress = 0.0;
                    animation.animate_forward = false;
                } else {
                    let Some(mut e) = commands.get_entity(hud) else {
                        return;
                    };
                    e.remove::<Animation>();
                }
            }
        } else if animation.animate_backward {
            transform.translation = animation
                .target_transform
                .translation
                .lerp(animation.initial_transform.translation, animation.progress);
            transform.rotation = animation
                .target_transform
                .rotation
                .lerp(animation.initial_transform.rotation, animation.progress);

            if 1.0 <= animation.progress {
                let Some(mut e) = commands.get_entity(hud) else {
                    return;
                };
                e.remove::<Animation>();
            }
        }
    }
}
