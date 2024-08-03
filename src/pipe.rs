
use bevy_ecs::{entity::Entity, query::QueryItem, system::{ParamSet, Query, SystemParamItem}};

use crate::{EntitySystem, filter::MatchesData};


type ESPipeFilter<A, B> = (
    MatchesData<<A as EntitySystem>::Data>,
    MatchesData<<B as EntitySystem>::Data>,
    <A as EntitySystem>::Filter,
    <B as EntitySystem>::Filter,
);

/// [`EntitySystem`] that pipes output of the first [`EntitySystem`] to the second [`EntitySystem`]
pub struct EntitySystemPipe<A: EntitySystem, B: EntitySystem<In = A::Out>> {
    a: A,
    b: B,
}

impl<A: EntitySystem, B: EntitySystem<In = A::Out>> EntitySystem
    for EntitySystemPipe<A, B>
{
    type Data = Entity;
    type Filter = ESPipeFilter<A, B>;
    type Param = ParamSet<
        'static,
        'static,
        (
            (
                Query<'static, 'static, A::Data, ESPipeFilter<A, B>>,
                A::Param,
            ),
            (
                Query<'static, 'static, B::Data, ESPipeFilter<A, B>>,
                B::Param,
            ),
        ),
    >;

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

        let result = A::run(&mut self.a, input, data_value, param);

        let (mut query, param) = set.p1();
        let data_value = query.get_mut(entity).unwrap();

        B::run(&mut self.b, result, data_value, param)
    }
}

/// Pipes output of the first system into second system. See [`EntitySystemPipe`]
#[inline]
pub fn entity_system_pipe<A: EntitySystem, B: EntitySystem<In = A::Out>>(
    a: A,
    b: B,
) -> EntitySystemPipe<A, B> {
    EntitySystemPipe { a, b }
}



