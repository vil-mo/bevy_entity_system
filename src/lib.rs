#![warn(clippy::undocumented_unsafe_blocks, missing_docs)]
//! This crate provides easy to use way to make systems that operate on a single entity.
//!
//! # Warning
//! Currently there is a bug, individual entity systems don't store their state.
//! State is being shared across all entities that are being iterated by the system that resulted from `into_system` call.
//! That means that `Local`, `EntityReader` and other `SystemParam`'s, that
//! rely on it's state property to be preserved between system runs, won't work correctly.
//! It is a huge footgun, so the work on fixing it is being done.
//!
//! After fix, the API will change, user would need to explicitly specify the entities this system is allowed to run on.
//! That means, you most probably would need to be able to mutate systems at 
//! runtime - something that bevy currently doesn't support. 
//! I also work on a crate that will be able to provide you with such functionality - it is not published yet.
//!
//! # Example
//!  
//! ```
//! # use bevy_ecs::prelude::*;
//! # use bevy_entity_system::prelude::*;
//! #[derive(Component)]
//! struct Count(i32);
//! 
//! #[derive(Component)]
//! struct MyMarkerComponent;
//!
//! fn my_entity_system(
//!     mut data: Data<&mut Count, With<MyMarkerComponent>>,
//!     mut commands: Commands
//! ) {
//!     data.item.0 += 1;
//!     commands.spawn(Count(10));
//! }
//!
//! fn my_entity_system_with_input(input: In<i32>, mut data: Data<&mut Count>) {
//!     data.item.0 += *input;
//! }
//!
//! bevy_ecs::system::assert_is_system(my_entity_system.into_system());
//! bevy_ecs::system::assert_is_system(my_entity_system_with_input.into_system());
//! ```

extern crate self as bevy_entity_system;

use bevy_ecs::{
    query::{QueryData, QueryFilter, QueryItem, ReadOnlyQueryData},
    system::{ReadOnlySystemParam, SystemParam, SystemParamItem},
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
/// # use bevy_entity_system::prelude::*;
/// # use bevy_ecs::prelude::*;
/// #[derive(Component)]
/// struct Count(i32);
/// 
/// #[derive(Component)]
/// struct MyMarkerComponent;
///
/// fn my_entity_system(
///     mut data: Data<&mut Count, With<MyMarkerComponent>>,
///     mut commands: Commands
/// ) {
///     data.item.0 += 1;
///     commands.spawn(Count(10));
/// }
///
/// fn my_entity_system_with_input(input: In<i32>, mut data: Data<&mut Count>) {
///     data.item.0 += *input;
/// }
/// 
/// # bevy_ecs::system::assert_is_system(my_entity_system.into_system());
/// # bevy_ecs::system::assert_is_system(my_entity_system_with_input.into_system());
/// ```
///
/// # Warning
/// Currently there is a bug, individual entity systems don't store their state.
/// State is being shared across all entities that are being iterated by the system that resulted from `into_system` call.
/// That means that `Local`, `EntityReader` and other `SystemParam`'s, that
/// rely on it's state property to be preserved between system runs, won't work correctly.
/// It is a huge footgun, so the work on fixing it is being done.
///
/// After fix, the API will change, user would need to explicitly specify the entities this system is allowed to run on.
/// That means, you most probably would need to be able to mutate systems at 
/// runtime - something that bevy currently doesn't support. 
/// I also work on a crate that will be able to provide you with such functionality - it is not published yet.
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

/// Implemented for [`EntitySystem`]s that only read data from the world
pub trait ReadOnlyEntitySystem:
    EntitySystem<Data: ReadOnlyQueryData, Param: ReadOnlySystemParam>
{
}

impl<T: EntitySystem<Data: ReadOnlyQueryData, Param: ReadOnlySystemParam>> ReadOnlyEntitySystem for T {}

/// Prelude module
pub mod prelude {
    pub use crate::{
        implementors::{AdapterEntitySystem, OptionalEntitySystem, PipeEntitySystem},
        into_entity_system::{EntitySystemIntoSystem, IntoEntitySystem},
        marked_entity_system::Data,
        EntitySystem, ReadOnlyEntitySystem,
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
