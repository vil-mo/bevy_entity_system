//! Convenience in working with [`EntitySystem`]s

use crate::{
    implementors::{adapt, entity_system_pipe, optional},
    marked_entity_system::{MarkedEntitySystem, MarkedEntitySystemRunner},
    prelude::{AdapterEntitySystem, OptionalEntitySystem, PipeEntitySystem},
    EntitySystem, ReadOnlyEntitySystem,
};
use bevy_ecs::system::{IntoSystem, ParamSet, Query, ReadOnlySystem, System};

/// Glue trait for convenience of working with [`EntitySystem`]s.
/// Everything that implements this trait can be converted to [`EntitySystem`].
pub trait IntoEntitySystem<In, Out, Marker>: Sized {
    /// Resulting [`EntitySystem`]
    type EntitySystem: EntitySystem<In = In, Out = Out>;

    /// Converts `Self` to [`EntitySystem`]
    fn into_entity_system(self) -> Self::EntitySystem;

    /// Converts [`EntitySystem`] to [`System`] that collects output of every entity system that is being run and returns result as an output.
    /// This implementation will iterate over all the entities in the world
    /// that can be run on by `Self::EntitySystem` every time the system is run
    /// and runs `EntitySystem` for them. Input will be cloned for every run of `EntitySystem`.
    /// `func` will be run for each run of `EntitySystem` passing in an output of that system and modifying output value each step.
    /// Initial value for output is it's [`Default`] value.
    ///
    /// ```
    /// # use bevy_ecs::prelude::*;
    /// # use bevy_entity_system::prelude::*;
    /// #[derive(Component)]
    /// struct Count(i32);
    ///
    /// fn get_count(data: Data<&Count>) -> i32 {
    ///     data.0
    /// }
    ///
    /// // When system being run, it will output sum of all values of entities with `Count` component in the world
    /// let system = get_count.into_system_with_output(|sum: &mut i32, count| *sum += count);
    /// 
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    fn into_system_with_output<T: Default + 'static>(
        self,
        mut func: impl FnMut(&mut T, Out) + Send + Sync + 'static,
    ) -> impl System<In = In, Out = T>
    where
        In: Clone + 'static,
    {
        let mut entity_system = self.into_entity_system();

        IntoSystem::into_system(
            move |input: bevy_ecs::system::In<In>,
                  mut query: Query<
                <Self::EntitySystem as EntitySystem>::Data,
                <Self::EntitySystem as EntitySystem>::Filter,
            >,
                  mut param: ParamSet<(<Self::EntitySystem as EntitySystem>::Param,)>| {
                let mut output = T::default();

                for data in query.iter_mut() {
                    let result = Self::EntitySystem::run(
                        &mut entity_system,
                        input.clone(),
                        data,
                        param.p0(),
                    );
                    func(&mut output, result);
                }

                output
            },
        )
    }

    /// Converts [`ReadOnlyEntitySystem`] to [`ReadOnlySystem`] that collects output of every entity system that is being run and returns result as an output.
    /// This implementation will iterate over all the entities in the world
    /// that can be run on by `Self::EntitySystem` every time the system is run
    /// and runs `EntitySystem` for them. Input will be cloned for every run of `EntitySystem`.
    /// `func` will be run for each run of `EntitySystem` passing in an output of that system and modifying output value each step.
    /// Initial value for output is it's [`Default`] value.
    fn into_read_only_system_with_output<T: Default + 'static>(
        self,
        mut func: impl FnMut(&mut T, Out) + Send + Sync + 'static,
    ) -> impl ReadOnlySystem<In = In, Out = T>
    where
        In: Clone + 'static,
        Self::EntitySystem: ReadOnlyEntitySystem,
    {
        let mut entity_system = self.into_entity_system();

        IntoSystem::into_system(
            move |input: bevy_ecs::system::In<In>,
                  mut query: Query<
                <Self::EntitySystem as EntitySystem>::Data,
                <Self::EntitySystem as EntitySystem>::Filter,
            >,
                  mut param: ParamSet<(<Self::EntitySystem as EntitySystem>::Param,)>| {
                let mut output = T::default();

                for data in query.iter_mut() {
                    let result = Self::EntitySystem::run(
                        &mut entity_system,
                        input.clone(),
                        data,
                        param.p0(),
                    );
                    func(&mut output, result);
                }

                output
            },
        )
    }


    /// See [`PipeEntitySystem`]
    #[inline]
    fn pipe<BMarker, BOut, B: IntoEntitySystem<Out, BOut, BMarker>>(
        self,
        other: B,
    ) -> PipeEntitySystem<Self::EntitySystem, B::EntitySystem> {
        entity_system_pipe(self.into_entity_system(), other.into_entity_system())
    }

    /// Maps output of the system to the new value.
    #[inline]
    fn map<F: Send + Sync + 'static + FnMut(Out) -> U, U>(
        self,
        func: F,
    ) -> AdapterEntitySystem<Self::EntitySystem, F> {
        adapt(self.into_entity_system(), func)
    }

    /// See [`OptionalEntitySystem`]
    #[inline]
    fn optional(self) -> OptionalEntitySystem<Self::EntitySystem> {
        optional(self.into_entity_system())
    }
}

impl<T: EntitySystem> IntoEntitySystem<T::In, T::Out, ()> for T {
    type EntitySystem = Self;
    #[inline]
    fn into_entity_system(self) -> Self::EntitySystem {
        self
    }
}

#[doc(hidden)]
pub struct IsMarkedEntitySystem;

impl<Marker: 'static, T: MarkedEntitySystem<Marker>>
    IntoEntitySystem<T::In, T::Out, (IsMarkedEntitySystem, Marker)> for T
{
    type EntitySystem = MarkedEntitySystemRunner<Marker, Self>;
    #[inline]
    fn into_entity_system(self) -> Self::EntitySystem {
        MarkedEntitySystemRunner::new(self)
    }
}

/// Trait exists because `=` in where clauses isn't allowed
pub trait EntitySystemIntoSystem<In: Clone + 'static, Marker>:
    IntoEntitySystem<In, (), Marker>
{
    /// Turns [`EntitySystem`] into [`System`]
    ///
    /// Using this implementation will output the system that iterates over all the entities in the world
    /// that can be run on by `<Self as IntoEntitySystem>::EntitySystem` every time the system is run.
    /// Input to the system will be cloned for every run of entity system
    fn into_system(self) -> impl System<In = In, Out = ()>;

    /// Turns [`ReadOnlyEntitySystem`] into [`ReadOnlySystem`]
    ///
    /// Using this implementation will output the system that iterates over all the entities in the world
    /// that can be run on by `<Self as IntoEntitySystem>::EntitySystem` every time the system is run.
    /// Input to the system will be cloned for every run of entity system
    fn into_read_only_system(self) -> impl ReadOnlySystem<In = In, Out = ()>
        where Self::EntitySystem: ReadOnlyEntitySystem;
}

impl<In: Clone + 'static, Marker, T: IntoEntitySystem<In, (), Marker>>
    EntitySystemIntoSystem<In, Marker> for T
{
    fn into_system(self) -> impl System<In = In, Out = ()> {
        let mut entity_system = self.into_entity_system();

        IntoSystem::into_system(
            move |input: bevy_ecs::system::In<In>,
                  mut query: Query<
                <T::EntitySystem as EntitySystem>::Data,
                <T::EntitySystem as EntitySystem>::Filter,
            >,
                  mut param: ParamSet<(<T::EntitySystem as EntitySystem>::Param,)>| {
                for data in query.iter_mut() {
                    T::EntitySystem::run(&mut entity_system, input.clone(), data, param.p0());
                }
            },
        )
    }

    fn into_read_only_system(self) -> impl ReadOnlySystem<In = In, Out = ()>
        where Self::EntitySystem: ReadOnlyEntitySystem {
        let mut entity_system = self.into_entity_system();

        IntoSystem::into_system(
            move |input: bevy_ecs::system::In<In>,
                  mut query: Query<
                <T::EntitySystem as EntitySystem>::Data,
                <T::EntitySystem as EntitySystem>::Filter,
            >,
                  mut param: ParamSet<(<T::EntitySystem as EntitySystem>::Param,)>| {
                for data in query.iter_mut() {
                    T::EntitySystem::run(&mut entity_system, input.clone(), data, param.p0());
                }
            },
        )
    }

}
