use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, DeriveInput, PredicateType};

fn bevy_entity_system_path() -> syn::Path {
    bevy_macro_utils::BevyManifest::default().get_path("bevy_entity_system")
}

fn bevy_ecs_path() -> syn::Path {
    bevy_macro_utils::BevyManifest::default().get_path("bevy_ecs")
}

/// Can be derived for `EntitySystem`s to have more convenient workflow with bevy ecosystem.
/// 
/// Using this implementation will output the system that iterates over all the entities
/// that match the `Query<Self::Data, Self::Filter>` every time the system is run.
/// Input to the system will be cloned for every run of entity system
#[proc_macro_derive(IntoSystem)]
pub fn derive_into_system_for_entity_system(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ecs_path = bevy_ecs_path();
    let entity_system_path = bevy_entity_system_path();

    let name = ast.ident;

    let mut generics = ast.generics;

    let temp_generics = generics.clone();
    let (_, ty_generics, _) = temp_generics.split_for_impl();

    let where_clause = generics.make_where_clause();
    where_clause
        .predicates
        .push(syn::WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty: syn::Type::Verbatim(quote!(#name #ty_generics)),
            colon_token: Default::default(),
            bounds: {
                let mut punctuated = Punctuated::new();
                punctuated.push(syn::TypeParamBound::Verbatim(quote!(
                    #entity_system_path::into_entity_system::IntoEntitySystem<In, (), __M>
                )));

                punctuated
            },
        }));

    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let punctuated_generics = &generics.params;

    TokenStream::from(quote! {
            impl<In: Clone, __M, #punctuated_generics>
                #ecs_path::system::IntoSystem<In, (), (#entity_system_path::into_system::IsEntitySystem, __M)>
                for #name #ty_generics
                #where_clause
            {
                type System = #ecs_path::system::FunctionSystem<
                    #entity_system_path::into_system::IsEntitySystem,
                    #entity_system_path::into_system::EntitySystemSystemParamFunction<
                        <#name #ty_generics as #entity_system_path::into_entity_system::IntoEntitySystem<In, (), __M>>::EntitySystem
                    >,
                >;

                #[inline]
                fn into_system(this: Self) -> Self::System {
                    #ecs_path::system::IntoSystem::into_system(
                        #entity_system_path::into_system::EntitySystemSystemParamFunction(
                            #entity_system_path::into_entity_system::IntoEntitySystem::into_entity_system(this)
                        ),
                    )
                }
            }
    })
}
