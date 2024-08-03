use bevy_ecs::{
    query::{QueryData, QueryFilter, QueryItem},
    system::{SystemParam, SystemParamItem},
};

pub mod entity_system;
pub mod entity_system_function;
pub mod filter;
pub mod into_entity_system;
pub mod pipe;

/// Trait implemented for all functions that can be used as [`System`](bevy_ecs::system::System)s
/// and operate on a single [`Entity`](bevy_ecs::entity::Entity).
/// 
/// 
pub trait EntitySystem: Send + Sync + 'static {
    /// [`QueryData`] to fetch from the world using [`Query`](bevy_ecs::system::Query) about the entity being iterated over.
    type Data: QueryData + 'static;
    /// Filtering of the entities that are allowed to be iterated over by this system.
    type Filter: QueryFilter + 'static;
    /// [`SystemParam`]'s of the system
    type Param: SystemParam + 'static;

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
        EntitySystem,
        filter::MatchesData,
        into_entity_system::{
            IntoEntitySystem, IntoEntitySystemImplementations, IntoEntitySystemIntoSystem,
        },
        pipe::EntitySystemPipe,
    };
}


#[cfg(test)]
mod tests {
    #[test]
    fn entity_system_test() {




    }
}