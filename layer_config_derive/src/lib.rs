use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(LayerConfig)]
pub fn layer_config_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let output = impl_layered_config(&ast);
    output.into()
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            return last_segment.ident == "Option";
        }
    }
    false
}

fn impl_layered_config(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = &ast.ident;
    let layered_config_internal_ident = format_ident!("{}LayeredConfigInternal", struct_name);

    // Note: We don't import LayeredConfig here - it's already in scope from layer_config crate
    // Using full path layer_config::LayeredConfig in the impl blocks avoids re-import issues

    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields,
            _ => unimplemented!("LayeredConfig only supports structs with named fields."),
        },
        _ => unimplemented!("LayeredConfig can only be derived for structs."),
    };

    let layered_config_opts_fields = fields.named.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;
        let option_ty = if is_option_type(ty) {
            quote! { #ty }
        } else {
            quote! { Option<#ty> }
        };

        let clap_attrs =
            field.attrs.iter().filter_map(
                |attr| {
                    if attr.path().is_ident("clap") {
                        Some(quote! { #attr })
                    } else {
                        None
                    }
                },
            );

        quote! {
            #(#clap_attrs)*
            pub #name: #option_ty,
        }
    });

    let merge_function = {
        let field_merges = fields.named.iter().map(|field| {
            let name = &field.ident;
            quote! {
                #name: rhs.#name.clone().or_else(|| lhs.#name.clone()),
            }
        });

        quote! {
            pub fn merge(lhs: &Self, rhs: &Self) -> Self {
                Self {
                    #(#field_merges)*
                }
            }
        }
    };

    let merge_final_function = {
        let field_resolutions = fields.named.iter().map(|field| {
            let name = &field.ident;
            quote! {
                #name: if cli_opts.#name.as_ref() != default_value_opts.#name.as_ref() {
                    cli_opts.#name.clone()
                } else {
                    precedence_opts.#name.clone()
                },
            }
        });

        quote! {
            pub fn merge_final(cli_opts: &Self, default_value_opts: &Self, precedence_opts: &Self) -> Self {
                Self {
                    #(#field_resolutions)*
                }
            }
        }
    };

    let from_env_function = {
        let env_assignments = fields.named.iter().map(|field| {
            let ident = &field.ident;
            let ident_str = ident.as_ref().unwrap().to_string().to_uppercase();
            let ty = &field.ty;
            let option_wrapped = is_option_type(ty);

            let env_var_assignment = if option_wrapped {
                quote! {
                    std::env::var(#ident_str).ok()
                }
            } else {
                quote! {
                    std::env::var(#ident_str).ok().and_then(|s| s.parse().ok())
                }
            };

            quote! {
                #ident: #env_var_assignment
            }
        });

        quote! {
            pub fn from_env() -> Self {
                Self {
                    #(#env_assignments),*
                }
            }
        }
    };

    let load_yaml_function = quote! {
        pub fn load_yaml(config_path: Option<&str>, default_value_opts: &Self) -> Self {
            if let Some(config_path) = config_path {
                if std::path::Path::new(config_path).exists() {
                    match std::fs::read_to_string(config_path) {
                        Ok(config_contents) => match serde_yaml::from_str(&config_contents) {
                            Ok(yml_opts) => yml_opts,
                            Err(_) => default_value_opts.clone(),
                        },
                        Err(_) => default_value_opts.clone(),
                    }
                } else {
                    default_value_opts.clone()
                }
            } else {
                default_value_opts.clone()
            }
        }
    };

    let layered_config_internal_impl = quote! {
        #[derive(Clone, Debug, Default, serde::Deserialize, clap::Parser)]
        #[serde(rename_all = "kebab-case")]
        struct #layered_config_internal_ident {
            #(#layered_config_opts_fields)*
        }

        impl #layered_config_internal_ident {
            #merge_function
            #merge_final_function
            #from_env_function
            #load_yaml_function
        }
    };

    let from_impl_fields = fields.named.iter().map(|field| {
        let name = &field.ident;
        quote! {
            #name: layered_config_internal.#name.take().unwrap_or_default()
        }
    });

    let from_impl = quote! {
        impl From<#layered_config_internal_ident> for #struct_name {
            fn from(mut layered_config_internal: #layered_config_internal_ident) -> Self {
                Self {
                    #(#from_impl_fields,)*
                }
            }
        }
    };

    let config_impl = {
        let has_config_field = fields.named.iter().any(|field| {
            if let Some(ident) = &field.ident {
                if ident == "config" {
                    if let syn::Type::Path(type_path) = &field.ty {
                        return type_path.path.is_ident("String");
                    }
                }
            }
            false
        });
        if has_config_field {
            quote! {
                impl layer_config::LayeredConfig for #struct_name {
                    fn resolve() -> Result<Self, Box<dyn std::error::Error>> {
                        let args: Vec<String> = std::env::args().collect();
                        #struct_name::resolve_from(args)
                    }
                    fn resolve_from<T: AsRef<[String]>>(args: T) -> Result<Self, Box<dyn std::error::Error>> {
                        let args: Vec<&str> = args.as_ref().iter().map(AsRef::as_ref).collect();
                        let default_value_opts = #layered_config_internal_ident::parse_from([] as [&str; 0]);
                        let cli_opts = #layered_config_internal_ident::parse_from(&args);
                        let yml_opts = #layered_config_internal_ident::load_yaml(cli_opts.config.as_deref(), &default_value_opts);
                        let precedence_opts = #layered_config_internal_ident::merge(&default_value_opts, &yml_opts);
                        let env_opts = #layered_config_internal_ident::from_env();
                        let precedence_opts = #layered_config_internal_ident::merge(&precedence_opts, &env_opts);
                        let final_opts = #layered_config_internal_ident::merge_final(&cli_opts, &default_value_opts, &precedence_opts);
                        Ok(final_opts.into())
                    }
                }
            }
        } else {
            quote! {
                impl layer_config::LayeredConfig for #struct_name {
                    fn resolve() -> Result<Self, Box<dyn std::error::Error>> {
                        let args: Vec<String> = std::env::args().collect();
                        #struct_name::resolve_from(args)
                    }
                    fn resolve_from<T: AsRef<[String]>>(args: T) -> Result<Self, Box<dyn std::error::Error>> {
                        let args: Vec<&str> = args.as_ref().iter().map(AsRef::as_ref).collect();
                        let default_value_opts = #layered_config_internal_ident::parse_from([] as [&str; 0]);
                        let cli_opts = #layered_config_internal_ident::parse_from(&args);
                        let env_opts = #layered_config_internal_ident::from_env();
                        let precedence_opts = #layered_config_internal_ident::merge(&default_value_opts, &env_opts);
                        let final_opts = #layered_config_internal_ident::merge_final(&cli_opts, &default_value_opts, &precedence_opts);
                        Ok(final_opts.into())
                    }
                }
            }
        }
    };

    quote! {
        #layered_config_internal_impl
        #from_impl
        #config_impl
    }
}
