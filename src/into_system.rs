//! Implementation of [`IntoSystem`]

use crate::EntitySystem;
use bevy_ecs::system::{lifetimeless::SQuery, ParamSet, SystemParamFunction, SystemParamItem};

pub use bevy_entity_system_macros::IntoSystem;

/// Marker for [`IntoSystem`] implementation
pub struct IsEntitySystem;

/// [`SystemParamFunction`], every time it's run, it iterates over all the
/// entities in the world that `T` can run on
/// and runs `T` for them. Input will be cloned for every run of `T`.
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
