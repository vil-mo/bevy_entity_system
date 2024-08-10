use bevy_ecs::system::{lifetimeless::SQuery, ParamSet, SystemParamFunction, SystemParamItem};

use crate::EntitySystem;

pub struct IsEntitySystem;
pub struct EntitySystemSystemParamFunction<T: EntitySystem<In: Clone, Out = ()>>(pub T);

impl<T: EntitySystem<In: Clone, Out = ()>> SystemParamFunction<IsEntitySystem>
    for EntitySystemSystemParamFunction<T>
{
    type In = T::In;
    type Out = ();
    type Param = (
        SQuery<T::Data, T::Filter>,
        ParamSet<'static, 'static, (T::Param,)>,
    );

    fn run(&mut self, input: Self::In, param_value: SystemParamItem<Self::Param>) -> Self::Out {
        let (mut query, mut param) = param_value;

        for data in query.iter_mut() {
            T::run(&mut self.0, input.clone(), data, param.p0());
        }
    }
}
