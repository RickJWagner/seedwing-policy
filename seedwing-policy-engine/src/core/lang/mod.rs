use crate::core::lang::and::And;
use crate::core::lang::or::Or;
use crate::core::lang::refine::Refine;
use crate::core::{json, Function, FunctionEvaluationResult};
use crate::lang::lir::Type;
use crate::lang::lir::{Bindings, InnerType};
use crate::package::Package;
use crate::runtime::{Output, RuntimeError};
use crate::runtime::{PackagePath, World};
use crate::value::{RationaleResult, RuntimeValue};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::str::from_utf8;
use std::sync::Arc;

mod and;
mod or;
mod refine;

pub fn package() -> Package {
    let mut pkg = Package::new(PackagePath::from_parts(vec!["lang"]));
    pkg.register_function("And".into(), And);
    pkg.register_function("Or".into(), Or);
    pkg.register_function("Refine".into(), Refine);
    pkg
}