use crate::{prelude::IntoEntitySystem, EntitySystem};
use bevy_ecs::{
    query::{QueryData, QueryFilter, QueryItem},
    system::{In, SystemParam, SystemParamItem},
};
use bevy_entity_system_macros::IntoSystem;
use bevy_utils::all_tuples;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(IntoSystem)]
pub struct MarkedEntitySystemRunner<Marker: 'static, T: MarkedEntitySystem<Marker>>(
    T,
    PhantomData<fn() -> Marker>,
);

impl<Marker: 'static, T: MarkedEntitySystem<Marker>> MarkedEntitySystemRunner<Marker, T> {
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

pub trait MarkedEntitySystem<Marker>: Send + Sync + 'static {
    type Data: QueryData + 'static;
    type Filter: QueryFilter + 'static;
    type Param: SystemParam + 'static;

    type In;
    type Out;

    /// Executes this system once. See [`System::run`] or [`System::run_unsafe`].
    fn run(
        &mut self,
        input: Self::In,
        data_value: QueryItem<Self::Data>,
        param_value: SystemParamItem<Self::Param>,
    ) -> Self::Out;
}

pub struct Data<'w, D: QueryData, F: QueryFilter = ()> {
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
    pub fn new(item: D::Item<'w>) -> Self {
        Data {
            item,
            marker: PhantomData,
        }
    }

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
