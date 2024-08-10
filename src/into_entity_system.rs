use bevy_ecs::system::{IntoSystem, ParamSet, Query, System};

use crate::{
    implementors::{adapt, entity_system_pipe, optional},
    marked_entity_system::{MarkedEntitySystem, MarkedEntitySystemRunner},
    EntitySystem,
};

pub trait IntoEntitySystem<In, Out, Marker>: Sized {
    type EntitySystem: EntitySystem<In = In, Out = Out>;

    fn into_entity_system(self) -> Self::EntitySystem;

    fn into_system_with_output<T: Default>(
        self,
        mut func: impl FnMut(&mut T, Out) + Send + Sync + 'static,
    ) -> impl System<In = In, Out = T>
    where
        In: Clone + 'static,
        T: 'static,
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

    #[inline]
    fn pipe<BMarker, BOut>(
        self,
        other: impl IntoEntitySystem<Out, BOut, BMarker>,
    ) -> impl EntitySystem<In = In, Out = BOut> {
        entity_system_pipe(self.into_entity_system(), other.into_entity_system())
    }

    #[inline]
    fn map<F: Send + Sync + 'static + FnMut(Out) -> U, U>(
        self,
        func: F,
    ) -> impl EntitySystem<In = In, Out = U> {
        adapt(self.into_entity_system(), func)
    }

    #[inline]
    fn optional(self) -> impl EntitySystem<In = In, Out = Result<Out, In>> {
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

pub trait EntitySystemIntoSystem<In: Clone + 'static, Marker>:
    IntoEntitySystem<In, (), Marker>
{
    fn into_system(self) -> impl System<In = In, Out = ()>;
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
}
