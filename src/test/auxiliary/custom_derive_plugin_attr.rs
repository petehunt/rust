// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// force-host

#![feature(plugin_registrar)]
#![feature(box_syntax)]
#![feature(rustc_private)]

extern crate syntax;
extern crate rustc;

use syntax::ast;
use syntax::attr::AttrMetaMethods;
use syntax::codemap::Span;
use syntax::ext::base::{Decorator, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic::{cs_fold, TraitDef, MethodDef, combine_substructure};
use syntax::ext::deriving::generic::{Substructure, Struct, EnumMatching};
use syntax::ext::deriving::generic::ty::{Literal, LifetimeBounds, Path, borrowed_explicit_self};
use syntax::parse::token;
use syntax::ptr::P;
use rustc::plugin::Registry;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("derive_TotalSum"),
        Decorator(box expand));
}

fn expand(cx: &mut ExtCtxt,
          span: Span,
          mitem: &ast::MetaItem,
          item: &ast::Item,
          push: &mut FnMut(P<ast::Item>)) {
    let trait_def = TraitDef {
        span: span,
        attributes: vec![],
        path: Path::new(vec!["TotalSum"]),
        additional_bounds: vec![],
        generics: LifetimeBounds::empty(),
        associated_types: vec![],
        methods: vec![
            MethodDef {
                name: "total_sum",
                generics: LifetimeBounds::empty(),
                explicit_self: borrowed_explicit_self(),
                args: vec![],
                ret_ty: Literal(Path::new_local("isize")),
                attributes: vec![],
                combine_substructure: combine_substructure(Box::new(totalsum_substructure)),
            },
        ],
    };

    trait_def.expand(cx, mitem, item, push)
}

// Mostly copied from syntax::ext::deriving::hash
/// Defines how the implementation for `trace()` is to be generated
fn totalsum_substructure(cx: &mut ExtCtxt, trait_span: Span,
                         substr: &Substructure) -> P<ast::Expr> {
    let fields = match *substr.fields {
        Struct(ref fs) | EnumMatching(_, _, ref fs) => fs,
        _ => cx.span_bug(trait_span, "impossible substructure")
    };

    fields.iter().fold(cx.expr_isize(trait_span, 0), |acc, ref item| {
        if item.attrs.iter().find(|a| a.check_name("ignore")).is_some() {
            acc
        } else {
            cx.expr_binary(item.span, ast::BiAdd, acc,
                           cx.expr_method_call(item.span,
                                               item.self_.clone(),
                                               substr.method_ident,
                                               Vec::new()))
        }
    })
}
