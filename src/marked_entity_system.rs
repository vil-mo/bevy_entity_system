//! Contains functionality to run [`EntitySystem`]s that should be marked in order to implement this trait
//! This is necessary to avoid conflicting implementations when implementing trait for rust functions

use crate::EntitySystem;
use bevy_ecs::{
    query::{QueryData, QueryFilter, QueryItem},
    system::{In, SystemParam, SystemParamFunction, SystemParamItem},
};
use bevy_entity_system_macros::IntoSystem;
use bevy_utils::all_tuples;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// Runs [`MarkedEntitySystem`]
#[derive(IntoSystem)]
pub struct MarkedEntitySystemRunner<Marker: 'static, T: MarkedEntitySystem<Marker>>(
    T,
    PhantomData<fn() -> Marker>,
);

impl<Marker: 'static, T: MarkedEntitySystem<Marker>> MarkedEntitySystemRunner<Marker, T> {
    /// Constructor
    #[inline]
    pub fn new(function: T) -> Self {
        MarkedEntitySystemRunner(function, PhantomData)
    }
}

impl<Marker: 'static, T: MarkedEntitySystem<Marker>> EntitySystem
    for MarkedEntitySystemRunner<Marker, T>
{
    type Data = T::Data;
    type Filter = T::Filter;
    type Param = T::Param;

    type In = T::In;
    type Out = T::Out;

    fn run(
        &mut self,
        input: Self::In,
        data_value: QueryItem<Self::Data>,
        param_value: SystemParamItem<Self::Param>,
    ) -> Self::Out {
        self.0.run(input, data_value, param_value)
    }
}

/// Trait implemented for all types that are defined by implementing a certain
/// trait defined in other crate and can be treated as [`EntitySystem`]s.
/// For example rust closures ([`FnMut`]) and [`SystemParamFunction`]. 
/// This is needed to avoid conflicting implementations.
/// 
/// [`MarkedEntitySystemRunner`] is used to run such entity systems
pub trait MarkedEntitySystem<Marker>: Send + Sync + 'static {
    /// First generic argument of the [`Data`] param of the function.
    /// Converted to [`EntitySystem::Data`]
    type Data: QueryData;
    /// Second generic argument of the [`Data`] param of the function
    type Filter: QueryFilter;
    /// [`SystemParam`]s of the function
    type Param: SystemParam;

    /// Input of the function. See [`In`]
    type In;
    /// Output of the function
    type Out;

    /// Executes this system once.
    fn run(
        &mut self,
        input: Self::In,
        data_value: QueryItem<Self::Data>,
        param_value: SystemParamItem<Self::Param>,
    ) -> Self::Out;
}

/// Marker of [`MarkedEntitySystem`] implementation for [`SystemParamFunction`]
pub struct IsSystemParamFunction;

impl<T: SystemParamFunction<M>, M> MarkedEntitySystem<(IsSystemParamFunction, M)> for T {
    type Data = ();
    type Filter = ();
    type Param = T::Param;

    type In = T::In;
    type Out = T::Out;

    fn run(
            &mut self,
            input: Self::In,
            _data_value: QueryItem<Self::Data>,
            param_value: SystemParamItem<Self::Param>,
        ) -> Self::Out {
        T::run(self, input, param_value)
    }
}

/// First (or second after [`In`]) parameter of any function that can be treated as [`EntitySystem`].
pub struct Data<'w, D: QueryData, F: QueryFilter = ()> {
    /// Item of the `QueryData`. What you get by calling `Query::get` or similar methods
    pub item: D::Item<'w>,
    marker: PhantomData<F>,
}

impl<'w, D: QueryData, F: QueryFilter> Deref for Data<'w, D, F> {
    type Target = D::Item<'w>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'w, D: QueryData, F: QueryFilter> DerefMut for Data<'w, D, F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<'w, D: QueryData, F: QueryFilter> Data<'w, D, F> {
    /// New instance of `Data` with provided item
    pub fn new(item: D::Item<'w>) -> Self {
        Data {
            item,
            marker: PhantomData,
        }
    }

    /// Converts `Data` into inner item. Basically the same as `let _ = data.item`
    #[inline]
    pub fn into_inner(self) -> D::Item<'w> {
        self.item
    }
}

macro_rules! impl_entity_system_function {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        impl<
            QData: QueryData,
            QFilter: QueryFilter,
            Out,
            Func: Send + Sync + 'static,
            $($param: SystemParam + 'static),*
        > MarkedEntitySystem<fn(Data<QData, QFilter>, $($param,)*) -> Out> for Func
        where
        for <'a> &'a mut Func:
                FnMut(Data<QData, QFilter>, $($param),*) -> Out +
                FnMut(Data<QData, QFilter>, $(SystemParamItem<$param>),*) -> Out,
                QData: 'static, QFilter: 'static, Out: 'static
        {
            type Data = QData;
            type Filter = QFilter;
            type Param = ($($param,)*);

            type In = ();
            type Out = Out;

            #[inline]
            fn run(&mut self, _input: (), data_value: QueryItem<QData>, param_value: SystemParamItem< ($($param,)*)>) -> Out {
                // Yes, this is strange, but `rustc` fails to compile this impl
                // without using this function. It fails to recognize that `func`
                // is a function, potentially because of the multiple impls of `FnMut`
                #[allow(clippy::too_many_arguments)]
                fn call_inner<QData: QueryData, QFilter: QueryFilter, Out, $($param,)*>(
                    mut f: impl FnMut(Data<QData, QFilter>, $($param,)*)->Out,
                    data: Data<QData, QFilter>,
                    $($param: $param,)*
                )->Out{
                    f(data, $($param,)*)
                }

                let ($($param,)*) = param_value;
                let data = Data::new(data_value);
                call_inner(self, data, $($param),*)
            }
        }



        #[allow(non_snake_case)]
        impl<
            QData: QueryData,
            QFilter: QueryFilter,
            Input,
            Out,
            Func: Send + Sync + 'static,
            $($param: SystemParam + 'static),*
        > MarkedEntitySystem<fn(In<Input>, Data<QData, QFilter>, $($param,)*) -> Out> for Func
        where
        for <'a> &'a mut Func:
                FnMut(In<Input>, Data<QData, QFilter>, $($param),*) -> Out +
                FnMut(In<Input>, Data<QData, QFilter>, $(SystemParamItem<$param>),*) -> Out,
                QData: 'static, QFilter: 'static, Out: 'static
        {
            type Data = QData;
            type Filter = QFilter;
            type Param = ($($param,)*);

            type In = Input;
            type Out = Out;

            #[inline]
            fn run(&mut self, input: Input, data_value: QueryItem<QData>, param_value: SystemParamItem< ($($param,)*)>) -> Out {
                // Yes, this is strange, but `rustc` fails to compile this impl
                // without using this function. It fails to recognize that `func`
                // is a function, potentially because of the multiple impls of `FnMut`
                #[allow(clippy::too_many_arguments)]
                fn call_inner<QData: QueryData, QFilter: QueryFilter, Input, Out, $($param,)*>(
                    mut f: impl FnMut(In<Input>, Data<QData, QFilter>, $($param,)*)->Out,
                    input: In<Input>,
                    data: Data<QData, QFilter>,
                    $($param: $param,)*
                )->Out{
                    f(input, data, $($param,)*)
                }

                let ($($param,)*) = param_value;
                let data = Data::new(data_value);
                call_inner(self, In(input), data, $($param),*)
            }
        }

    };
}

all_tuples!(impl_entity_system_function, 0, 16, F);
