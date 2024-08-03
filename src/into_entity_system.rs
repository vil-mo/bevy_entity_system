use bevy_ecs::system::{IntoSystem, ParamSet, Query, System};

use crate::{
    EntitySystem,
    entity_system_function::{EntitySystemFunction, FunctionEntitySystem},
    pipe::entity_system_pipe,
};

pub trait IntoEntitySystem<In, Out, Marker> {
    type EntitySystem: EntitySystem<In = In, Out = Out>;
    fn into_entity_system(self) -> Self::EntitySystem;
}

impl<T: EntitySystem> IntoEntitySystem<T::In, T::Out, ()> for T {
    type EntitySystem = Self;
    #[inline]
    fn into_entity_system(self) -> Self::EntitySystem {
        self
    }
}





struct IsEntitySystemFunction;

impl<Marker: 'static, T: EntitySystemFunction<Marker>>
    IntoEntitySystem<T::In, T::Out, (IsEntitySystemFunction, Marker)> for T
{
    type EntitySystem = FunctionEntitySystem<Self, Marker>;
    #[inline]
    fn into_entity_system(self) -> Self::EntitySystem {
        FunctionEntitySystem::new(self)
    }
}

pub trait IntoEntitySystemIntoSystem<Marker>: IntoEntitySystem<(), (), Marker> {
    fn into_system(self) -> impl System;
}

impl<Marker, T: IntoEntitySystem<(), (), Marker>> IntoEntitySystemIntoSystem<Marker> for T {
    fn into_system(self) -> impl System {
        let mut entity_system = self.into_entity_system();

        IntoSystem::into_system(
            move |mut query: Query<
                <T::EntitySystem as EntitySystem>::Data,
                <T::EntitySystem as EntitySystem>::Filter,
            >,
                  mut param: ParamSet<(<T::EntitySystem as EntitySystem>::Param,)>| {
                for data in query.iter_mut() {
                    T::EntitySystem::run(&mut entity_system, (), data, param.p0());
                }
            },
        )
    }
}

pub trait IntoEntitySystemImplementations<AIn, AOut, AMarker>:
    IntoEntitySystem<AIn, AOut, AMarker>
{
    fn pipe<BMarker, BOut>(
        self,
        other: impl IntoEntitySystem<AOut, BOut, BMarker>,
    ) -> impl EntitySystem<In = AIn, Out = BOut>;
}

impl<AIn, AOut, Marker, T: IntoEntitySystem<AIn, AOut, Marker>>
    IntoEntitySystemImplementations<AIn, AOut, Marker> for T
{
    #[inline]
    fn pipe<BMarker, BOut>(
        self,
        other: impl IntoEntitySystem<AOut, BOut, BMarker>,
    ) -> impl EntitySystem<In = AIn, Out = BOut> {
        entity_system_pipe(self.into_entity_system(), other.into_entity_system())
    }
}
