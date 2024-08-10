//#![warn(clippy::undocumented_unsafe_blocks, missing_docs)]
//! Crate adds functionality for the systems that

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
///
/// Every entity system that is function
pub trait EntitySystem: Send + Sync + 'static {
    /// [`QueryData`] to fetch from the world using [`Query`](bevy_ecs::system::Query) about the entity being iterated over.
    type Data: QueryData;
    /// Filtering of the entities that are allowed to be iterated over by this system.
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

pub mod prelude {
    pub use crate::{
        implementors::{AdapterEntitySystem, OptionalEntitySystem, PipeEntitySystem},
        into_entity_system::IntoEntitySystem,
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
