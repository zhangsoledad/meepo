#![recursion_limit = "128"]

extern crate proc_macro;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::fold::Fold;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Eq;
use syn::{Block, FnArg, ItemFn, Pat, Type};

#[derive(Clone)]
struct Options {
    maxsize: u64,
    fn_arg: Vec<FnArg>,
    ret_ty: Type,
}

struct Attr {
    pub key: syn::Ident,
    _eq: Eq,
    pub value: syn::LitInt,
}

impl syn::parse::Parse for Attr {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

struct Args {
    attrs: Vec<Attr>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let vars = Punctuated::<Attr, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            attrs: vars.into_iter().collect(),
        })
    }
}

impl Fold for Options {
    fn fold_block(&mut self, i: Block) -> Block {
        let arg_idents = arg_idents(&self.fn_arg);
        let pat: Vec<Pat> = arg_idents.iter().map(|a| a.0.clone()).collect();
        let ty: Vec<Type> = arg_idents.iter().map(|a| a.1.clone()).collect();
        let arg = quote! {
            (#(#pat),*)
        };
        let key = quote! {
            (#(#ty),*)
        };

        let ret_ty = self.ret_ty.clone();

        parse_quote! {
            {
                use std::collections::HashMap;
                use std::cell::RefCell;
                thread_local! {
                    static MEEPO: RefCell<HashMap<#key, #ret_ty>> = RefCell::new(HashMap::default());
                }
                MEEPO.with(|meepo| {
                    let mut meepo = meepo.borrow_mut();
                    let result = meepo.get(&#arg).cloned();
                    if result.is_none() {
                        let result = #i;
                        meepo.insert(#arg, result.clone());
                        result
                    } else {
                        result.unwrap()
                    }
                })
            }
        }
    }
}

fn arg_idents(args: &[FnArg]) -> Vec<(Pat, Type)> {
    let mut idents = vec![];
    for arg in args {
        if let syn::FnArg::Captured(ref cap) = arg {
            idents.push((cap.pat.clone(), cap.ty.clone()));
        }
    }
    idents
}

#[proc_macro_attribute]
pub fn meepo(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    // Parse the list of variables the user wanted to print.
    let args = parse_macro_input!(args as Args);

    let ret_ty = if let syn::ReturnType::Type(_, ref ty) = input.decl.output {
        ty.clone()
    } else {
        panic!("meepo is not applicable function without return");
    };

    let mut option = Options {
        maxsize: args.attrs[0].value.value(),
        fn_arg: input.decl.inputs.iter().cloned().collect(),
        ret_ty: *ret_ty,
    };

    // Use a syntax tree traversal to transform the function body.
    let output = option.fold_item_fn(input);

    // Hand the resulting function body back to the compiler.
    TokenStream::from(quote!(#output))
}

// #[plugin_registrar]
// pub fn registrar(reg: &mut Registry) {
//     reg.register_syntax_extension(
//         Symbol::intern("meepo"),
//         MultiModifier(Box::new(meepo_expand)),
//     );
// }

// fn get_options(cx: &mut ExtCtxt, meta: &MetaItem) -> Options {
//     let mut options = Options::default();
//     if let List(ref v) = meta.node {
//         for i in v {
//             if let NestedMetaItemKind::MetaItem(ref mi) = i.node {
//                 if let NameValue(ref s) = mi.node {
//                     if mi.name() == "maxsize" {
//                         if let Str(ref maxsize, _) = s.node {
//                             match maxsize.to_string().parse() {
//                                 Ok(maxsize) => options.maxsize = maxsize,
//                                 Err(e) => cx.span_warn(
//                                     i.span,
//                                     &format!("Invalid option {} {:?}", mi.name(), e),
//                                 ),
//                             }
//                         }
//                     } else {
//                         cx.span_warn(i.span, &format!("Invalid option {}", mi.name()));
//                     }
//                 } else {
//                     cx.span_warn(i.span, &format!("Invalid option {}", mi.name()));
//                 }
//             }
//         }
//     }
//     options
// }

// fn meepo_expand(
//     cx: &mut ExtCtxt,
//     sp: Span,
//     meta: &MetaItem,
//     annotatable: Annotatable,
// ) -> Annotatable {
//     let options = get_options(cx, meta);

//     match annotatable {
//         Annotatable::Item(item) => {
//             let res = match item.node {
//                 Fn(..) => {
//                     let new_item = expand_function(cx, options, &item);
//                     cx.item(item.span, item.ident, item.attrs.clone(), new_item)
//                         .map(|mut it| {
//                             it.vis = item.vis.clone();
//                             it
//                         })
//                 }
//                 _ => {
//                     cx.span_err(sp, "meepo is only permissible on functions");
//                     item.clone()
//                 }
//             };
//             Annotatable::Item(res)
//         }
//         Annotatable::ImplItem(item) => {
//             let new_item = expand_impl_method(cx, options, &item);
//             Annotatable::ImplItem(P(ImplItem {
//                 node: new_item,
//                 attrs: vec![],
//                 ..(*item).clone()
//             }))
//         }
//         _ => {
//             cx.span_err(sp, "meepo is not applicable this items");
//             annotatable.clone()
//         }
//     }
// }

// fn expand_function(cx: &mut ExtCtxt, options: Options, item: &P<Item>) -> ItemKind {
//     let name = &&*item.ident.name.as_str();

//     if let Fn(ref decl, fnheader, ref generics, ref block) = item.node {
//         let idents = arg_idents(cx, &**decl);
//         let new_block = expand_block(cx, options, name, block.clone(), idents, &**decl);
//         Fn(decl.clone(), fnheader, generics.clone(), new_block)
//     } else {
//         panic!("Expected a function")
//     }
// }

// fn expand_impl_method(cx: &mut ExtCtxt, options: Options, item: &ImplItem) -> ImplItemKind {
//     let name = &*item.ident.name.as_str();

//     if let ImplItemKind::Method(ref sig, ref block) = item.node {
//         let idents = arg_idents(cx, &sig.decl);
//         let new_block = expand_block(cx, options, name, block.clone(), idents, &sig.decl);
//         ImplItemKind::Method(sig.clone(), new_block)
//     } else {
//         panic!("Expected method");
//     }
// }

// fn arg_idents(cx: &mut ExtCtxt, decl: &FnDecl) -> Vec<Ident> {
//     fn extract_idents(cx: &mut ExtCtxt, pat: &ast::PatKind, idents: &mut Vec<Ident>) {
//         match *pat {
//             PatKind::Paren(..)
//             | PatKind::Wild
//             | PatKind::TupleStruct(_, _, None)
//             | PatKind::Lit(_)
//             | PatKind::Range(..)
//             | PatKind::Path(..) => (),
//             PatKind::Ident(_, id, _) => {
//                 if id.name.as_str() != "self" {
//                     idents.push(id);
//                 }
//             }
//             PatKind::TupleStruct(_, ref v, _) | PatKind::Tuple(ref v, _) => {
//                 for p in v {
//                     extract_idents(cx, &p.node, idents);
//                 }
//             }
//             PatKind::Struct(_, ref v, _) => {
//                 for p in v {
//                     extract_idents(cx, &p.node.pat.node, idents);
//                 }
//             }
//             PatKind::Slice(ref v1, ref opt, ref v2) => {
//                 for p in v1 {
//                     extract_idents(cx, &p.node, idents);
//                 }
//                 if let &Some(ref p) = opt {
//                     extract_idents(cx, &p.node, idents);
//                 }
//                 for p in v2 {
//                     extract_idents(cx, &p.node, idents);
//                 }
//             }
//             PatKind::Box(ref p) | PatKind::Ref(ref p, _) => extract_idents(cx, &p.node, idents),
//             PatKind::Mac(ref m) => {
//                 let sp = m.node.path.span;
//                 cx.span_warn(sp, "meepo ignores pattern macros in function arguments");
//             }
//         }
//     }
//     let mut idents = vec![];
//     for arg in decl.inputs.iter() {
//         extract_idents(cx, &arg.pat.node, &mut idents);
//     }
//     idents
// }

// fn expand_block(
//     cx: &mut ExtCtxt,
//     options: Options,
//     name: &str,
//     block: P<Block>,
//     idents: Vec<Ident>,
//     decl: &FnDecl,
// ) -> P<Block> {
//     let args: Vec<TokenTree> = idents
//         .iter()
//         .map(|ident| vec![token::Token::from_ast_ident(ident.clone())])
//         .collect::<Vec<_>>()
//         .join(&token::Comma)
//         .into_iter()
//         .map(|t| TokenTree::Token(source_map::DUMMY_SP, t))
//         .collect();

//     let mut arg_fmt = vec![];
//     for ident in idents.iter() {
//         arg_fmt.push(format!("{}: {{:?}}", ident))
//     }
//     let arg_fmt_str = &*arg_fmt.join(", ");

//     let new_block = quote_expr!(cx, unsafe {
//         let mut __meepo_closure = move || $block;
//         let __meepo_result = __meepo_closure();
//         __meepo_result
//     });
//     cx.block_expr(new_block)
// }
