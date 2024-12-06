//! Implements the `chunk` function from the WDL standard library.

use std::sync::Arc;

use wdl_ast::Diagnostic;

use super::CallContext;
use super::Function;
use super::Signature;
use crate::Array;
use crate::Value;
use crate::diagnostics::function_call_failed;

/// Given an array and a length `n`, splits the array into consecutive,
/// non-overlapping arrays of n elements.
///
/// If the length of the array is not a multiple `n` then the final sub-array
/// will have length(array) % `n` elements.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#-chunk
fn chunk(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 2);

    let array = context.arguments[0]
        .value
        .as_array()
        .expect("argument should be an array");

    let size = context.arguments[1]
        .value
        .as_integer()
        .expect("argument should be an integer");

    if size < 0 {
        return Err(function_call_failed(
            "chunk",
            "chunk size cannot be negative",
            context.arguments[1].span,
        ));
    }

    let element_ty = context
        .types()
        .type_definition(
            context
                .return_type
                .as_compound()
                .expect("type should be compound")
                .definition(),
        )
        .as_array()
        .expect("type should be an array")
        .element_type();

    let elements = array
        .elements()
        .chunks(size as usize)
        .map(|chunk| {
            Array::new_unchecked(element_ty, Arc::new(Vec::from_iter(chunk.iter().cloned()))).into()
        })
        .collect();

    Ok(Array::new_unchecked(context.return_type, Arc::new(elements)).into())
}

/// Gets the function describing `chunk`.
pub const fn descriptor() -> Function {
    Function::new(const { &[Signature::new("(Array[X], Int) -> Array[Array[X]]", chunk)] })
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use wdl_ast::version::V1;

    use crate::v1::test::TestEnv;
    use crate::v1::test::eval_v1_expr;

    #[test]
    fn chunk() {
        let mut env = TestEnv::default();

        let value = eval_v1_expr(&mut env, V1::Two, "chunk([], 10)").unwrap();
        assert_eq!(value.as_array().unwrap().len(), 0);

        let value = eval_v1_expr(&mut env, V1::Two, "chunk([1, 2, 3, 4, 5], 1)").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_array()
                    .unwrap()
                    .elements()
                    .iter()
                    .map(|v| v.as_integer().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(elements, [[1], [2], [3], [4], [5]]);

        let value = eval_v1_expr(&mut env, V1::Two, "chunk([1, 2, 3, 4, 5], 2)").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_array()
                    .unwrap()
                    .elements()
                    .iter()
                    .map(|v| v.as_integer().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(elements, [[1, 2].as_slice(), &[3, 4], &[5]]);

        let value = eval_v1_expr(&mut env, V1::Two, "chunk([1, 2, 3, 4, 5], 3)").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_array()
                    .unwrap()
                    .elements()
                    .iter()
                    .map(|v| v.as_integer().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(elements, [[1, 2, 3].as_slice(), &[4, 5]]);

        let value = eval_v1_expr(&mut env, V1::Two, "chunk([1, 2, 3, 4, 5], 4)").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_array()
                    .unwrap()
                    .elements()
                    .iter()
                    .map(|v| v.as_integer().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(elements, [[1, 2, 3, 4].as_slice(), &[5]]);

        let value = eval_v1_expr(&mut env, V1::Two, "chunk([1, 2, 3, 4, 5], 5)").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_array()
                    .unwrap()
                    .elements()
                    .iter()
                    .map(|v| v.as_integer().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(elements, [[1, 2, 3, 4, 5]]);

        let value = eval_v1_expr(&mut env, V1::Two, "chunk([1, 2, 3, 4, 5], 10)").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_array()
                    .unwrap()
                    .elements()
                    .iter()
                    .map(|v| v.as_integer().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(elements, [[1, 2, 3, 4, 5]]);

        let diagnostic = eval_v1_expr(&mut env, V1::Two, "chunk([1, 2, 3], -10)").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `chunk` failed: chunk size cannot be negative"
        );
    }
}