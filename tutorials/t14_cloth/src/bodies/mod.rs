mod ball;
mod softbody;
mod cloth;

pub use ball::*;
pub use softbody::*;
pub use cloth::*;

use bevy::prelude::*;

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
pub struct Velocity(pub Vec3);


