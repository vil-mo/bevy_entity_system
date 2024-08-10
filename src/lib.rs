#![warn(clippy::undocumented_unsafe_blocks, missing_docs)]
//! This crate provides easy to use way to make systems that operate on a single entity.
//!
//! ```rust
//! #[derive(Component)]
//! struct MyMarkerComponent;
//!
//! fn my_entity_system(
//!     data: Data<&mut Transform, With<MyMarkerComponent>>,
//!     mut commands: Commands
//! ) {
//!     *data.item += 10;
//!     commands.spawn(Transform::from_translation(data.item.translation));
//! }
//!
//! fn my_entity_system_with_input(input: In<Vec2>, data: Data<&mut Transform>) {
//!     *data.item += input;
//! }
//!
//! app.add_systems(Update, (
//!     my_entity_system.into_system(),
//!     my_entity_system_with_input.into_system()
//! ));
//! ```

extern crate self as bevy_entity_system;

use bevy_ecs::{
    query::{QueryData, QueryFilter, QueryItem},
    system::{SystemParam, SystemParamItem},
};

pub mod data_match;
pub mod implementors;
pub mod into_entity_system;
pub mod into_system;
pub mod marked_entity_system;

/// Trait implemented for all functions that can be used as [`System`](bevy_ecs::system::System)s
/// and operate on a single [`Entity`](bevy_ecs::entity::Entity).
/// Such system can only be run for entities that match it's [`Data`](EntitySystem::Data) and [`Filter`](EntitySystem::Filter)
///
/// Every entity system that is function that has [`Data`](crate::marked_entity_system::Data)
/// as a first (or second after [`In`](bevy_ecs::system::In)) parameter
/// ```
/// # use crate::prelude::*;
/// # use bevy_ecs::prelude::*;
/// #[derive(Component)]
/// struct MyMarkerComponent;
///
/// fn my_entity_system(
///     data: Data<&mut Transform, With<MyMarkerComponent>>,
///     mut commands: Commands
/// ) {
///     *data.item += 10;
///     commands.spawn(Transform::from_translation(data.item.translation));
/// }
///
/// fn my_entity_system_with_input(input: In<Vec2>, data: Data<&mut Transform>) {
///     *data.item += input;
/// }
/// ```
///
/// # Custom implementation
/// If you have a custom implementation of the trait, it's highly recommended to also
/// derive [`IntoSystem`](crate::prelude::macros::IntoSystem) trait
/// # Note
/// Since [`IntoSystem`](bevy_ecs::system::IntoSystem) is defined in `bevy_ecs` crate, it's impossible to implement it for the rust functions.
/// Use [`into_system`](crate::into_entity_system::EntitySystemIntoSystem::into_system) or
/// [`into_system_with_output`](crate::into_entity_system::IntoEntitySystem::into_system_with_output)
/// instead or convert to one of the type that implements such trait.
pub trait EntitySystem: Send + Sync + 'static {
    /// [`QueryData`] to get from the world for the entity that system is run on
    type Data: QueryData;
    /// Filtering of the entities that are allowed to be run on by this system
    type Filter: QueryFilter;
    /// [`SystemParam`]'s of the system
    type Param: SystemParam;

    /// Input of the system
    type In;
    /// Output of the system
    type Out;

    /// Executes this system once.
    fn run(
        &mut self,
        input: Self::In,
        data_value: QueryItem<Self::Data>,
        param_value: SystemParamItem<Self::Param>,
    ) -> Self::Out;
}

/// Prelude module
pub mod prelude {
    pub use crate::{
        implementors::{AdapterEntitySystem, OptionalEntitySystem, PipeEntitySystem},
        into_entity_system::{EntitySystemIntoSystem, IntoEntitySystem},
        marked_entity_system::Data,
        EntitySystem,
    };
    pub use bevy_entity_system_macros as macros;
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;

    use crate::prelude::*;

    #[test]
    fn entity_system_test() {
        #[derive(Component)]
        struct Count(u32);

        fn increment_count(data: Data<&mut Count>) -> u32 {
            let mut count = data.into_inner();

            count.0 += 1;

            count.0
        }

        let mut world = World::new();
        world.spawn(Count(0));
        let entity_with_10 = world.spawn(Count(10)).id();

        let system = world.register_system(
            increment_count.into_system_with_output(|val: &mut u32, step| *val += step),
        );

        assert_eq!(world.run_system(system).unwrap(), 12);

        assert_eq!(world.run_system(system).unwrap(), 14);
        world.spawn(Count(20));

        assert_eq!(world.run_system(system).unwrap(), 37);
        world.despawn(entity_with_10);
        assert_eq!(world.run_system(system).unwrap(), 26);
    }
}
