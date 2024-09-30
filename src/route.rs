#![allow(unused)]

pub mod server {
    use std::io::Write;
    use std::path::Path;
    use std::fs::read_to_string;
    use std::fs::File;
    use quote::TokenStreamExt;
    use quote::{quote, ToTokens};
    use proc_macro2::TokenStream;
    use convert_case::{Case, Casing};

    const PROPS_STRUCT_NAME: &'static str = "Props";
    const RENDER_FN_NAME: &'static str = "render";

    #[derive(Debug, Clone)]
    pub enum RustType {
        Int32,
        Int64,
        UInt32,
        UInt64,
        Float32,
        Float64,
        Boolean,
    }

    impl RustType {
        fn as_str(&self) -> &'static str {
            match self {
                RustType::Int32 => "i32",
                RustType::Int64 => "i64",
                RustType::UInt32 => "u32",
                RustType::UInt64 => "u64",
                RustType::Float32 => "f32",
                RustType::Float64 => "f64",
                RustType::Boolean => "bool",
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum Prop {
        Boolean{ name: String },
        String{ name: String },
        Number{ name: String, ty: RustType },
    }

    #[derive(Debug, Clone)]
    pub struct RustProp(Prop);
    impl ToTokens for RustProp {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match &self.0 {
                Prop::String{ name } => { 
                    let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                    let a = quote! { #ident: String }.to_tokens(tokens);
                },

                Prop::Number{ name, ty } => { 
                    let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                    let ty = syn::Ident::new(ty.as_str(), proc_macro2::Span::call_site());
                    let a = quote! { #ident: #ty }.to_tokens(tokens);
                },

                Prop::Boolean{ name } => { 
                    let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                    let a = quote! {  #ident: bool  }.to_tokens(tokens);
                },
                _ => {},
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct TypescriptProp(Prop);
    impl ToTokens for TypescriptProp {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match &self.0 {
                Prop::String{ name } => { 
                    let ident = syn::Ident::new(&name.to_case(Case::Camel), proc_macro2::Span::call_site());
                    let a = quote! { export declare const #ident: string; }.to_tokens(tokens);
                },

                Prop::Number{ name, .. } => { 
                    let ident = syn::Ident::new(&name.to_case(Case::Camel), proc_macro2::Span::call_site());
                    let a = quote! { export declare const #ident: number; }.to_tokens(tokens);
                },

                Prop::Boolean{ name } => { 
                    let ident = syn::Ident::new(&name.to_case(Case::Camel), proc_macro2::Span::call_site());
                    let a = quote! {  export declare const #ident: boolean;  }.to_tokens(tokens);
                },
                _ => {},
            }
        }
    }

    pub struct Route {
        pub props: Vec<Prop>,
    }

    impl Route {
        pub fn from_file(path: &Path) -> Result<Self, ()> {
            let source_code = read_to_string(path)
                .unwrap();

            let tokens = syn::parse_file(&source_code)
                .unwrap();

            let mut prop_fields: Vec<Prop> = Vec::new();
            for item in &tokens.items {
                match &item {
                    syn::Item::Struct(item_struct) => {
                        if item_struct.ident == PROPS_STRUCT_NAME {
                            for field in &item_struct.fields {
                                match &field.ty {
                                    syn::Type::Path(type_path) => {
                                        let type_ident = &type_path.path.segments.get(&type_path.path.segments.len() - 1).unwrap().ident.to_string();
                                        match type_ident.as_str() {
                                            "String" => { prop_fields.push(Prop::String { name: field.ident.clone().unwrap().to_string() }); },

                                            "bool" => { prop_fields.push(Prop::Boolean { name: field.ident.clone().unwrap().to_string() }); },

                                            "i8" | "i16" | "i32" | "i64" => {
                                                prop_fields.push(Prop::Number {
                                                    name: field.ident.clone().unwrap().to_string(),
                                                    ty: RustType::Int32,
                                                });
                                            },

                                            "u8" | "u16" | "u32" | "u64" => {
                                                prop_fields.push(Prop::Number {
                                                    name: field.ident.clone().unwrap().to_string(),
                                                    ty: RustType::UInt32,
                                                });
                                            },

                                            "f32" | "f64" => {
                                                prop_fields.push(Prop::Number {
                                                    name: field.ident.clone().unwrap().to_string(),
                                                    ty: RustType::Float32,
                                                });
                                            },

                                            _ => {},
                                        }
                                    }
                                    _ => {},
                                }
                            }
                        }
                    },

                    _ => {},
                }
            }

            Ok(Self { props: prop_fields.clone(), })
        }

        pub fn write_rust_module(&self, path: &Path) {
            let rust_props = self.props.iter().map(|p| { RustProp(p.clone()) });
            let tokens = quote! {
                #[derive(Debug, Clone)]
                pub struct Props {
                    #(#rust_props),*
                }
            };
            let tokens_string = tokens.to_string();
            let mut file = File::create(path)
                .unwrap();
            file.write_all(tokens_string.as_bytes());
        }

        pub fn write_js_declarations(&self, path: &Path) {
            let typescript_props = self.props.iter().map(|p| { TypescriptProp(p.clone()) });
            let tokens = quote! { #(#typescript_props)* };
            let tokens_string = tokens.to_string();
            let mut file = File::create(path)
                .unwrap();
            file.write_all(tokens_string.as_bytes());
        }
    }
}
