use bevy_ecs::{component::Component, query::{QueryData, ReadOnlyQueryData, WorldQuery}};

pub trait SFQueryData: QueryData<ReadOnly: ShrinkFetch> + ShrinkFetch {}

impl<T: QueryData<ReadOnly: ShrinkFetch> + ShrinkFetch> SFQueryData for T {}

pub trait SFReadOnlyQueryData: ReadOnlyQueryData + ShrinkFetch {}

impl<T: ReadOnlyQueryData + ShrinkFetch> SFReadOnlyQueryData for T {}


/// Made to overcome the fact associated types aren't covariant.
/// 
/// Analogue of [`shrink`](WorldQuery::shrink) for [`WorldQuery::Fetch`].
pub trait ShrinkFetch: WorldQuery {
    fn shrink_fetch<'wlong: 'wshort, 'wshort>(item: Self::Fetch<'wlong>) -> Self::Fetch<'wshort>;
}



impl <T: Component> ShrinkFetch for &'_ T {
    fn shrink_fetch<'wlong: 'wshort, 'wshort>(item: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        item
    }
}

impl <T: Component> ShrinkFetch for &'_ mut T {
    fn shrink_fetch<'wlong: 'wshort, 'wshort>(item: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        item
    }
}


