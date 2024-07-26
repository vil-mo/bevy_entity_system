use bevy_ecs::{
    archetype::Archetype,
    component::{ComponentId, Components, Tick},
    entity::Entity,
    query::{FilteredAccess, QueryData, ReadOnlyQueryData, WorldQuery},
    storage::{Table, TableRow},
    world::{unsafe_world_cell::UnsafeWorldCell, World},
};

pub type QueryDataItem<'w, D> = <D as WorldQuery>::Item<'w>;

/// Allows disjointed access to query data
///
/// ```
/// # use bevy_ecs::prelude::*;
/// # use bevy_data_set::DataSet;
/// #
/// # #[derive(Component)]
/// # struct Health;
/// #
/// # fn do_thing(_: &Health) {}
/// # fn do_thing_mut(_: &mut Health) {}
/// // Given the following system
/// fn fancy_system(mut query: Query<DataSet<(&mut Health, &Health)>>) {
///     for mut set in query.iter_mut() {
///         do_thing_mut(set.d0());
///         do_thing(set.d1());
///     }
/// }
/// # bevy_ecs::system::assert_is_system(fancy_system);
/// ```
pub struct DataSet<'w, T: QueryData> {
    items: T::Item<'w>,
}

impl<'w, D0: QueryData, D1: QueryData> DataSet<'w, (D0, D1)> {
    /// Gets exclusive access to the 1st parameter in this [`DataSet`].
    /// No other parameters may be accessed while this one is active
    pub fn d0(&mut self) -> &mut QueryDataItem<'w, D0> {
        &mut self.items.0
    }

    /// Gets exclusive access to the 2nd parameter in this [`DataSet`].
    /// No other parameters may be accessed while this one is active
    pub fn d1(&mut self) -> &mut QueryDataItem<'w, D1> {
        &mut self.items.1
    }
}

unsafe impl<'w, D0: QueryData, D1: QueryData> QueryData for DataSet<'w, (D0, D1)> {
    type ReadOnly = DataSet<'w, (D0::ReadOnly, D1::ReadOnly)>;
}

unsafe impl<'w, D0: ReadOnlyQueryData, D1: ReadOnlyQueryData> ReadOnlyQueryData
    for DataSet<'w, (D0, D1)>
{
}



/// SAFETY:
/// For each [`QueryData`] in the set, their respective [`update_component_access`] gets called inside [`update_component_access`] function.
/// [`DataSet`] is a conjunction, so [`matches_component_set`] is also a conjuction of [`QueryData`] in the set.
/// 
///
/// [`matches_component_set`]: Self::matches_component_set
/// [`update_component_access`]: Self::update_component_access
unsafe impl<'__w, D0: QueryData, D1: QueryData> WorldQuery for DataSet<'__w, (D0, D1)> {
    type Item<'w> = DataSet<'w, (D0, D1)>;
    type Fetch<'w> = (D0::Fetch<'w>, D1::Fetch<'w>);
    type State = (D0::State, D1::State);

    /// Dense if all sets are dense
    const IS_DENSE: bool = D0::IS_DENSE && D1::IS_DENSE;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        let DataSet { items: (d0, d1) } = item;
        DataSet {
            items: (D0::shrink(d0), D1::shrink(d1)),
        }
    }

    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        state: &Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Fetch<'w> {
        (
            D0::init_fetch(world.clone(), &state.0, last_run, this_run),
            D1::init_fetch(world.clone(), &state.1, last_run, this_run),
        )
    }

    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        state: &Self::State,
        archetype: &'w Archetype,
        table: &'w Table,
    ) {
        D0::set_archetype(&mut fetch.0, &state.0, archetype, table);
        D1::set_archetype(&mut fetch.1, &state.1, archetype, table);
    }

    unsafe fn set_table<'w>(fetch: &mut Self::Fetch<'w>, state: &Self::State, table: &'w Table) {
        D0::set_table(&mut fetch.0, &state.0, table);
        D1::set_table(&mut fetch.1, &state.1, table);
    }

    fn set_access(state: &mut Self::State, access: &FilteredAccess<ComponentId>) {
        D0::set_access(&mut state.0, access);
        D1::set_access(&mut state.1, access);
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Self::Item<'w> {
        DataSet {
            items: (
                D0::fetch(&mut fetch.0, entity, table_row),
                D1::fetch(&mut fetch.1, entity, table_row),
            ),
        }
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        // Updating access of empty [`FilteredAccess`] so then it can be used for conjunction
        let mut access0: FilteredAccess<ComponentId> = FilteredAccess::default();
        D0::update_component_access(&state.0, &mut access0);
        // Making sure each individual member of the set doesn't conflict with other query access
        D0::update_component_access(&state.0, &mut access.clone());


        // Updating access of empty [`FilteredAccess`] so then it can be used for conjunction
        let mut access1 = FilteredAccess::default();
        D1::update_component_access(&state.1, &mut access1);
        // Making sure each individual member of the set doesn't conflict with other query access
        D1::update_component_access(&state.1, &mut access.clone());


        // Extending access with conjunction of the accesses of all the members of the set
        access.extend(&access0);
        access.extend(&access1);
    }

    fn init_state(world: &mut World) -> Self::State {
        (D0::init_state(world), D1::init_state(world))
    }

    fn get_state(components: &Components) -> Option<Self::State> {
        let Some(d0) = D0::get_state(components) else {
            return None;
        };

        let Some(d1) = D1::get_state(components) else {
            return None;
        };

        Some((d0, d1))
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(bevy_ecs::component::ComponentId) -> bool,
    ) -> bool {
        D0::matches_component_set(&state.0, set_contains_id)
            && D1::matches_component_set(&state.1, set_contains_id)
    }
}



#[cfg(test)]
mod tests {

    #[test]
    fn my_test() {
        use super::DataSet;
        use bevy_ecs::prelude::*;

        #[derive(Component)]
        struct Health(i32);

        fn assert_health(health: &Health, value: i32) {
            assert_eq!(health.0, value);
        }
        fn multiply_health(health: &mut Health) {
            health.0 *= 2;
        }

        // Given the following system
        fn fancy_system(mut query: Query<DataSet<(&mut Health, &Health)>>) {
            for mut set in query.iter_mut() {
                assert_health(set.d1(),30);

                multiply_health(set.d0());

                assert_health(set.d1(), 60);
            }
        }

        let mut world = World::new();

        world.spawn(Health(30));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(fancy_system);
        schedule.run(&mut world);
    }
}
