use bevy::prelude::*;

/// Removes all entities with specified component with their children
pub fn remove_all_with<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for e in entities.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn set_state<S, const NS: u8>(mut state: ResMut<NextState<S>>)
where
    S: States,
    u8: IntoState<S>,
{
    state.set(<u8 as IntoState<S>>::into_state(NS));
}

pub trait IntoState<S: States> {
    fn into_state(self) -> S;
}

#[macro_export]
macro_rules! impl_into_state {
    ($t: tt) => {
        impl IntoState<$t> for u8 {
            fn into_state(self) -> $t {
                unsafe { std::mem::transmute(self) }
            }
        }
    };
}
