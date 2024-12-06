//! Implements the `unzip` function from the WDL standard library.

use std::sync::Arc;

use wdl_ast::Diagnostic;

use super::CallContext;
use super::Function;
use super::Signature;
use crate::Array;
use crate::Pair;
use crate::Value;

/// Creates a Pair of Arrays, the first containing the elements from the left
/// members of an Array of Pairs, and the second containing the right members.
///
/// If the array is empty, a pair of empty arrays is returned.
///
/// This is the inverse of the zip function.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#unzip
fn unzip(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 1);

    let array = context.arguments[0]
        .value
        .as_array()
        .expect("argument should be an array");

    let pair_ty = context
        .types()
        .type_definition(
            context
                .return_type
                .as_compound()
                .expect("type should be compound")
                .definition(),
        )
        .as_pair()
        .expect("type should be a pair");

    let left_ty = pair_ty.left_type();
    debug_assert!(
        context
            .types()
            .type_definition(
                left_ty
                    .as_compound()
                    .expect("type should be compound")
                    .definition(),
            )
            .as_array()
            .is_some(),
        "left type should be an array"
    );

    let right_ty = pair_ty.right_type();
    debug_assert!(
        context
            .types()
            .type_definition(
                right_ty
                    .as_compound()
                    .expect("type should be compound")
                    .definition(),
            )
            .as_array()
            .is_some(),
        "right type should be an array"
    );

    let mut left = Vec::with_capacity(array.len());
    let mut right = Vec::with_capacity(array.len());
    for v in array.elements() {
        let p = v.as_pair().expect("element should be a pair");
        left.push(p.left().clone());
        right.push(p.right().clone());
    }

    Ok(Pair::new_unchecked(
        context.return_type,
        Arc::new(Array::new_unchecked(left_ty, Arc::new(left)).into()),
        Arc::new(Array::new_unchecked(right_ty, Arc::new(right)).into()),
    )
    .into())
}

/// Gets the function describing `unzip`.
pub const fn descriptor() -> Function {
    Function::new(
        const {
            &[Signature::new(
                "(Array[Pair[X, Y]]) -> Pair[Array[X], Array[Y]]",
                unzip,
            )]
        },
    )
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use wdl_ast::version::V1;

    use crate::v1::test::TestEnv;
    use crate::v1::test::eval_v1_expr;

    #[test]
    fn unzip() {
        let mut env = TestEnv::default();

        let value = eval_v1_expr(&mut env, V1::One, "unzip([])")
            .unwrap()
            .unwrap_pair();
        assert_eq!(value.left().as_array().unwrap().len(), 0);
        assert_eq!(value.right().as_array().unwrap().len(), 0);

        let value = eval_v1_expr(&mut env, V1::One, "unzip([(1, 'a'), (2, 'b'), (3, 'c')])")
            .unwrap()
            .unwrap_pair();
        let left: Vec<_> = value
            .left()
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_integer().unwrap())
            .collect();
        assert_eq!(left, [1, 2, 3]);
        let right: Vec<_> = value
            .right()
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_string().unwrap().as_str())
            .collect();
        assert_eq!(right, ["a", "b", "c"]);
    }
}