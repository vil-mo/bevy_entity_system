use crate::{data_match::DataMatch, EntitySystem};
use bevy_ecs::{
    entity::Entity,
    query::QueryItem,
    system::{lifetimeless::SQuery, ParamSet, SystemParamItem},
};

type SParamSet<P> = ParamSet<'static, 'static, P>;

type ESPipeFilter<A, B> = (
    DataMatch<<A as EntitySystem>::Data>,
    <A as EntitySystem>::Filter,
    DataMatch<<B as EntitySystem>::Data>,
    <B as EntitySystem>::Filter,
);
type ESPipeParam<A, B> = SParamSet<(
    (
        SQuery<<A as EntitySystem>::Data, ESPipeFilter<A, B>>,
        <A as EntitySystem>::Param,
    ),
    (
        SQuery<<B as EntitySystem>::Data, ESPipeFilter<A, B>>,
        <B as EntitySystem>::Param,
    ),
)>;

/// [`EntitySystem`] that pipes output of the first [`EntitySystem`] to the second [`EntitySystem`]
pub struct PipeEntitySystem<A: EntitySystem, B: EntitySystem<In = A::Out>>(A, B);

impl<A: EntitySystem, B: EntitySystem<In = A::Out>> EntitySystem for PipeEntitySystem<A, B> {
    type Data = Entity;
    type Filter = ESPipeFilter<A, B>;
    type Param = ESPipeParam<A, B>;

    type In = A::In;
    type Out = B::Out;

    fn run(
        &mut self,
        input: Self::In,
        entity: QueryItem<Self::Data>,
        mut set: SystemParamItem<Self::Param>,
    ) -> Self::Out {
        let (mut query, param) = set.p0();
        let data_value = query.get_mut(entity).unwrap();

        let result = A::run(&mut self.0, input, data_value, param);

        let (mut query, param) = set.p1();
        let data_value = query.get_mut(entity).unwrap();

        B::run(&mut self.1, result, data_value, param)
    }
}

/// Pipes output of the first system into second system. See [`EntitySystemPipe`]
#[inline]
pub fn entity_system_pipe<A: EntitySystem, B: EntitySystem<In = A::Out>>(
    a: A,
    b: B,
) -> PipeEntitySystem<A, B> {
    PipeEntitySystem(a, b)
}

pub struct OptionalEntitySystem<T: EntitySystem>(T);

impl<T: EntitySystem> EntitySystem for OptionalEntitySystem<T> {
    type Data = Entity;
    type Filter = ();
    type Param = (SQuery<T::Data, T::Filter>, T::Param);

    type In = T::In;
    type Out = Result<T::Out, Self::In>;

    #[inline]
    fn run(
        &mut self,
        input: Self::In,
        data_value: QueryItem<Self::Data>,
        param_value: SystemParamItem<Self::Param>,
    ) -> Self::Out {
        let entity = data_value;
        let (mut query, param) = param_value;

        let x = if let Ok(data) = query.get_mut(entity) {
            Ok(self.0.run(input, data, param))
        } else {
            Err(input)
        };
        x
    }
}

#[inline]
pub fn optional<T: EntitySystem>(system: T) -> OptionalEntitySystem<T> {
    OptionalEntitySystem(system)
}

pub trait Adapt<S: EntitySystem>: Send + Sync + 'static {
    type In;
    type Out;

    // Required method
    fn adapt(
        &mut self,
        input: Self::In,
        run_system: impl FnOnce(<S as EntitySystem>::In) -> <S as EntitySystem>::Out,
    ) -> Self::Out;
}

impl<F, S, Out> Adapt<S> for F
where
    S: EntitySystem,
    F: Send + Sync + 'static + FnMut(S::Out) -> Out,
{
    type In = S::In;
    type Out = Out;

    #[inline]
    fn adapt(&mut self, input: S::In, run_system: impl FnOnce(S::In) -> S::Out) -> Out {
        self(run_system(input))
    }
}

pub struct AdapterEntitySystem<T: EntitySystem, Func: Adapt<T>> {
    system: T,
    func: Func,
}

impl<T: EntitySystem, Func: Adapt<T>> EntitySystem for AdapterEntitySystem<T, Func> {
    type Data = T::Data;
    type Filter = T::Filter;
    type Param = T::Param;

    type In = Func::In;
    type Out = Func::Out;

    #[inline]
    fn run(
        &mut self,
        input: Self::In,
        data_value: QueryItem<Self::Data>,
        param_value: SystemParamItem<Self::Param>,
    ) -> Self::Out {
        self.func.adapt(input, |input| {
            self.system.run(input, data_value, param_value)
        })
    }
}

#[inline]
pub fn adapt<T: EntitySystem, A: Adapt<T>>(system: T, func: A) -> AdapterEntitySystem<T, A> {
    AdapterEntitySystem { system, func }
}
