use crate::{core::ArcStr, db::api::view::BoxedIter, prelude::Prop, python::types::repr::Repr};
use chrono::NaiveDateTime;
use itertools::Itertools;
use num::cast::AsPrimitive;
use pyo3::prelude::*;
use std::{i64, iter::Sum};

pub(crate) trait MeanExt<V>: Iterator<Item = V>
where
    V: AsPrimitive<f64> + Sum<V>,
{
    fn mean(self) -> f64
    where
        Self: Sized,
    {
        let mut count: usize = 0;
        let sum: V = self.inspect(|_| count += 1).sum();

        if count > 0 {
            sum.as_() / (count as f64)
        } else {
            0.0
        }
    }
}

impl<I: ?Sized + Iterator<Item = V>, V: AsPrimitive<f64> + Sum<V>> MeanExt<V> for I {}

py_float_iterable!(Float64Iterable, f64);

py_numeric_iterable!(U64Iterable, u64);
py_nested_numeric_iterable!(NestedU64Iterable, u64, U64Iterable, OptionU64Iterable);

py_iterable!(OptionU64U64Iterable, Option<(u64, u64)>);
py_ordered_iterable!(U64U64Iterable, (u64, u64));
py_nested_ordered_iterable!(NestedU64U64Iterable, (u64, u64), OptionU64U64Iterable);

py_iterable!(OptionU64Iterable, Option<u64>, Option<u64>);
_py_ord_max_min_methods!(OptionU64Iterable, Option<u64>);

py_iterable!(PropIterable, Prop, Prop);
py_iterable_comp!(PropIterable, Prop, PropIterableCmp);

py_numeric_iterable!(I64Iterable, i64);
py_nested_numeric_iterable!(NestedI64Iterable, i64, I64Iterable, OptionI64Iterable);

py_iterable!(OptionI64Iterable, Option<i64>);
_py_ord_max_min_methods!(OptionI64Iterable, Option<i64>);
py_iterable!(OptionOptionI64Iterable, Option<Option<i64>>);
_py_ord_max_min_methods!(OptionOptionI64Iterable, Option<Option<i64>>);

py_nested_ordered_iterable!(
    NestedOptionI64Iterable,
    Option<i64>,
    OptionOptionI64Iterable
);

py_numeric_iterable!(UsizeIterable, usize);
py_iterable_comp!(UsizeIterable, usize, UsizeIterableCmp);

py_ordered_iterable!(OptionUsizeIterable, Option<usize>);
py_nested_numeric_iterable!(
    NestedUsizeIterable,
    usize,
    UsizeIterable,
    OptionUsizeIterable
);

py_iterable!(BoolIterable, bool);
py_nested_iterable!(NestedBoolIterable, bool);

py_iterable!(StringIterable, String);
py_iterable!(OptionArcStringIterable, Option<ArcStr>);
py_nested_iterable!(NestedOptionArcStringIterable, Option<ArcStr>);
py_nested_iterable!(NestedStringIterable, String);

py_iterable!(StringVecIterable, Vec<String>);
py_nested_iterable!(NestedStringVecIterable, Vec<String>);

py_iterable!(ArcStringVecIterable, Vec<ArcStr>);
py_iterable_comp!(ArcStringVecIterable, Vec<ArcStr>, ArcStringVecIterableCmp);
py_nested_iterable!(NestedArcStringVecIterable, Vec<ArcStr>);
py_iterable_comp!(
    NestedArcStringVecIterable,
    ArcStringVecIterableCmp,
    NestedArcStringVecIterableCmp
);

py_iterable!(I64VecIterable, Vec<i64>);
py_iterable_comp!(I64VecIterable, Vec<i64>, I64VecIterableCmp);
py_nested_iterable!(NestedI64VecIterable, Vec<i64>);
py_iterable_comp!(
    NestedI64VecIterable,
    I64VecIterableCmp,
    NestedI64VecIterableCmp
);

py_iterable!(OptionNaiveDateTimeIterable, Option<NaiveDateTime>);
py_nested_iterable!(NestedNaiveDateTimeIterable, Option<NaiveDateTime>);

py_nested_iterable!(NestedOptionStringIterable, Option<String>);
