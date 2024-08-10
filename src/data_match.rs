use std::marker::PhantomData;

use bevy_ecs::{
    component::{ComponentId, Components},
    query::{FilteredAccess, QueryData, QueryFilter, WorldQuery},
    world::World,
};

/// Filter that imitates `QueryData` being part of the query without read and write access.
/// Only adds `With` and `Without` access of the `QueryData` to the access of the system.
///
/// Useful in generic contexts.
///
/// `Query<(&mut Transform, T)>` and
/// `Query<&mut Transform, MatchesData<T>>`
/// will iterate over the same entities except system with second query
/// can be run in parallel with other systems accessing `T`, and
/// if `T` accesses `Transform` it won't create conflicts
pub struct DataMatch<D: QueryData>(PhantomData<D>);

/// SAFETY:
/// `update_component_access` does not add any accesses.
/// This is sound because `fetch` does not access any components.
/// `update_component_access` adds a `With` and `Without` filters of `D`.
/// This is sound because `matches_component_set` returns result of call to `D`'s `matches_component_set`.
unsafe impl<D: QueryData> WorldQuery for DataMatch<D> {
    type Item<'a> = ();
    type Fetch<'a> = ();
    type State = D::State;

    fn shrink<'wlong: 'wshort, 'wshort>(_: Self::Item<'wlong>) -> Self::Item<'wshort> {}

    unsafe fn init_fetch<'w>(
        _world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
        _state: &Self::State,
        _last_run: bevy_ecs::component::Tick,
        _this_run: bevy_ecs::component::Tick,
    ) -> Self::Fetch<'w> {
    }

    const IS_DENSE: bool = D::IS_DENSE;

    unsafe fn set_archetype<'w>(
        _fetch: &mut Self::Fetch<'w>,
        _state: &Self::State,
        _archetype: &'w bevy_ecs::archetype::Archetype,
        _table: &'w bevy_ecs::storage::Table,
    ) {
    }

    unsafe fn set_table<'w>(
        _fetch: &mut Self::Fetch<'w>,
        _state: &Self::State,
        _table: &'w bevy_ecs::storage::Table,
    ) {
    }

    unsafe fn fetch<'w>(
        _fetch: &mut Self::Fetch<'w>,
        _entity: bevy_ecs::entity::Entity,
        _table_row: bevy_ecs::storage::TableRow,
    ) -> Self::Item<'w> {
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        let mut data_access = FilteredAccess::default();
        D::update_component_access(state, &mut data_access);

        for index in data_access.with_filters() {
            access.and_with(index);
        }
        for index in data_access.without_filters() {
            access.and_without(index);
        }
    }

    fn init_state(world: &mut World) -> Self::State {
        D::init_state(world)
    }

    fn get_state(components: &Components) -> Option<Self::State> {
        D::get_state(components)
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        D::matches_component_set(state, set_contains_id)
    }
}

impl<D: QueryData> QueryFilter for DataMatch<D> {
    const IS_ARCHETYPAL: bool = true;

    unsafe fn filter_fetch(
        _fetch: &mut Self::Fetch<'_>,
        _entity: bevy_ecs::entity::Entity,
        _table_row: bevy_ecs::storage::TableRow,
    ) -> bool {
        true
    }
}
