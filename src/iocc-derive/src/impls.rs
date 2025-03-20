use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    visit_mut::{self, VisitMut},
    AngleBracketedGenericArguments, Attribute, Error as SynError, FnArg, GenericArgument, Ident,
    ImplItem, ImplItemFn, ItemImpl, Meta, PatType, Path, PathArguments, Result as SynResult,
    ReturnType, Signature, Type, TypePath,
};

use crate::attrs::AttributeData;

#[derive(Debug)]
struct ConstructorData {
    self_type: TypePath,
    identifier: Ident,
    arguments: Vec<PatType>,
    return_type: ReturnTypeData,
}

#[derive(Debug)]
enum ReturnTypeData {
    Infallible,
    Result { error_type: TypePath },
}

struct AttributeRemovalVisitor;

impl AttributeRemovalVisitor {
    fn is_inject_attribute(attr: &Attribute) -> bool {
        if let Meta::Path(path) = &attr.meta {
            if path.segments.first().is_some_and(|s| s.ident == "inject") {
                return true;
            }
        }
        false
    }
}

impl VisitMut for AttributeRemovalVisitor {
    fn visit_attributes_mut(&mut self, attrs: &mut Vec<Attribute>) {
        attrs.retain(|attr| !Self::is_inject_attribute(attr));
        attrs
            .iter_mut()
            .for_each(|attr| visit_mut::visit_attribute_mut(self, attr));
    }
}

pub fn expand_implementation(
    impls: TokenStream,
    attr_data: AttributeData,
) -> SynResult<TokenStream2> {
    let mut impls = match syn::parse::<ItemImpl>(impls) {
        Ok(impls) => impls,
        Err(err) => {
            return Err(SynError::new(
                err.span(),
                "`#[component]` should be annotated on the `impl` block",
            ))
        }
    };

    let self_type = get_self_type(&impls)?;
    let signature = get_constructor_signature(&impls.items, impls.span())?;
    let ctor_data = parse_constructor(self_type, signature)?;

    let expanded = expand_component_implementation(ctor_data, attr_data)?;

    let mut visitor = AttributeRemovalVisitor;
    visitor.visit_item_impl_mut(&mut impls);

    Ok(quote! {
        #impls
        #expanded
    })
}

fn get_self_type(impls: &ItemImpl) -> SynResult<TypePath> {
    if let Type::Path(ty) = impls.self_ty.as_ref() {
        Ok(ty.clone())
    } else {
        Err(SynError::new(impls.self_ty.span(), "invalid self type"))
    }
}

fn get_constructor_signature(items: &[ImplItem], impl_span: Span) -> SynResult<Signature> {
    let ctors: Vec<_> = items
        .iter()
        .filter_map(filter_and_map_item_fn)
        .filter(is_annotated_with_inject)
        .collect();

    let signature = if ctors.len() > 1 {
        return Err(SynError::new(
            impl_span,
            "only one associated function can be annotated with `#[inject]`",
        ));
    } else if let Some(&ctor) = ctors.first() {
        ctor.sig.clone()
    } else {
        return Err(SynError::new(
            impl_span,
            "no associated function is annotated with `#[inject]`",
        ));
    };

    if let Some(FnArg::Receiver(rec)) = signature.inputs.first() {
        return Err(SynError::new(
            rec.span(),
            "method is not allowed to be annotated with `#[inject]`",
        ));
    }

    Ok(signature)
}

fn filter_and_map_item_fn(item: &ImplItem) -> Option<&ImplItemFn> {
    if let ImplItem::Fn(impl_fn) = item {
        Some(impl_fn)
    } else {
        None
    }
}

fn is_annotated_with_inject(item_fn: &&ImplItemFn) -> bool {
    item_fn.attrs.iter().any(|attr| {
        let content = attr.meta.to_token_stream().to_string();
        &content == "inject"
    })
}

fn parse_constructor(self_type: TypePath, signature: Signature) -> SynResult<ConstructorData> {
    let identifier = signature.ident;
    let arguments = parse_constructor_arguments(signature.inputs);
    let return_type = parse_constructor_return_type(signature.output, &self_type)?;

    Ok(ConstructorData {
        self_type,
        identifier,
        arguments,
        return_type,
    })
}

fn parse_constructor_arguments(inputs: Punctuated<FnArg, Comma>) -> Vec<PatType> {
    inputs
        .into_iter()
        .map(|arg| {
            if let FnArg::Typed(arg) = arg {
                arg
            } else {
                unreachable!("a constructor should not have a receiver argument");
            }
        })
        .collect::<Vec<_>>()
}

fn parse_constructor_return_type(
    output: ReturnType,
    self_type: &TypePath,
) -> Result<ReturnTypeData, SynError> {
    let ReturnType::Type(_, return_type) = output else {
        return Err(SynError::new(
            output.span(),
            "a constructor's return type should be `Self` or `Result<Self, E>`",
        ));
    };
    let Type::Path(return_type) = *return_type else {
        return Err(SynError::new(
            return_type.span(),
            "a constructor's return type should be `Self` or `Result<Self, E>`",
        ));
    };

    let segmengs = &return_type.path.segments;

    let return_type = if &return_type == self_type {
        ReturnTypeData::Infallible
    } else if segmengs.len() == 1 && segmengs.first().unwrap().ident == "Self" {
        ReturnTypeData::Infallible
    } else if segmengs.len() == 1 && segmengs.first().unwrap().ident == "Result" {
        parse_result_return_type(&segmengs.first().unwrap().arguments, self_type)?
    } else if segmengs.len() == 3
        && segmengs.get(0).unwrap().ident == "std"
        && segmengs.get(1).unwrap().ident == "result"
        && segmengs.get(2).unwrap().ident == "Result"
    {
        parse_result_return_type(&segmengs.get(2).unwrap().arguments, self_type)?
    } else {
        return Err(SynError::new(
            return_type.span(),
            "a constructor's return type should be `Self` or `Result<Self, E>`",
        ));
    };
    Ok(return_type)
}

fn parse_result_return_type(
    type_args: &PathArguments,
    self_type: &TypePath,
) -> SynResult<ReturnTypeData> {
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
        args: type_args, ..
    }) = type_args
    else {
        return Err(SynError::new(
            type_args.span(),
            "a constructor's return type should be `Self` or `Result<Self, E>`",
        ));
    };

    if type_args.len() == 2 {
        let GenericArgument::Type(Type::Path(first_type)) = type_args.get(0).unwrap() else {
            return Err(SynError::new(
                type_args.span(),
                "a constructor's return type should be `Self` or `Result<Self, E>`",
            ));
        };
        let GenericArgument::Type(Type::Path(second_path)) = type_args.get(1).unwrap() else {
            return Err(SynError::new(
                type_args.span(),
                "a constructor's return type should be `Self` or `Result<Self, E>`",
            ));
        };

        let segments = &first_type.path.segments;

        if first_type == self_type {
            Ok(ReturnTypeData::Result {
                error_type: second_path.clone(),
            })
        } else if segments.first().is_some_and(|s| s.ident == "Self") {
            Ok(ReturnTypeData::Result {
                error_type: second_path.clone(),
            })
        } else {
            Err(SynError::new(
                type_args.span(),
                "a constructor's return type should be `Self` or `Result<Self, E>`",
            ))
        }
    } else {
        Err(SynError::new(
            type_args.span(),
            "a constructor's return type should be `Self` or `Result<Self, E>`",
        ))
    }
}

fn expand_component_implementation(
    ctor_data: ConstructorData,
    attr_data: AttributeData,
) -> SynResult<TokenStream2> {
    let self_type = &ctor_data.self_type;
    let constructor = &ctor_data.identifier;

    let associated_type_constructed = if let AttributeData::Full { output_type, .. } = &attr_data {
        let output_type = syn::parse_str::<TypePath>(output_type).unwrap();
        quote! { type Constructed = #output_type; }
    } else {
        quote! { type Constructed = #self_type; }
    };

    let associated_type_error =
        if let ReturnTypeData::Result { error_type } = &ctor_data.return_type {
            quote! { type Error = #error_type; }
        } else {
            quote! { type Error = std::convert::Infallible; }
        };

    let get_dep_statements = ctor_data
        .arguments
        .iter()
        .map(|arg| {
            let dep = arg.pat.as_ref();
            quote! { let #dep = injector.get(key::of())?; }
        })
        .collect::<TokenStream2>();

    let dep_args = ctor_data
        .arguments
        .iter()
        .map(|arg| {
            let dep = arg.pat.as_ref();
            quote! { #dep, }
        })
        .collect::<TokenStream2>();

    let wire_deps = if let ReturnTypeData::Infallible = &ctor_data.return_type {
        quote! { Ok(Ok(#self_type::#constructor(#dep_args))) }
    } else {
        quote! { Ok(#self_type::#constructor(#dep_args)) }
    };

    let post_process_body = if let AttributeData::Full { post_processor, .. } = &attr_data {
        let post_processor = syn::parse_str::<Path>(post_processor).unwrap();
        quote! { #post_processor(self) }
    } else {
        quote! { self }
    };

    Ok(quote! {
        impl iocc::provider::component::Component for #self_type {
            #associated_type_constructed
            #associated_type_error

            fn construct<I>(injector: &I) -> Result<Result<Self, Self::Error>, InjectorError>
            where
                I: TypedInjector + ?Sized
            {
                #get_dep_statements
                #wire_deps
            }

            fn post_process(self) -> Self::Constructed {
                #post_process_body
            }
        }
    })
}
