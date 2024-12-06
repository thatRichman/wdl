//! Implements the `read_lines` function from the WDL standard library.

use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::Arc;

use anyhow::Context;
use wdl_analysis::stdlib::STDLIB as ANALYSIS_STDLIB;
use wdl_analysis::types::PrimitiveTypeKind;
use wdl_ast::Diagnostic;

use super::CallContext;
use super::Function;
use super::Signature;
use crate::Array;
use crate::PrimitiveValue;
use crate::Value;
use crate::diagnostics::function_call_failed;

/// Reads each line of a file as a String, and returns all lines in the file as
/// an Array[String].
///
/// Trailing end-of-line characters (\r and \n) are removed from each line.
///
/// The order of the lines in the returned Array[String] is the order in which
/// the lines appear in the file.
///
/// If the file is empty, an empty array is returned.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#read_lines
fn read_lines(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(ANALYSIS_STDLIB.array_string_type()));

    let path = context.cwd().join(
        context
            .coerce_argument(0, PrimitiveTypeKind::File)
            .unwrap_file()
            .as_str(),
    );

    let file = fs::File::open(&path)
        .with_context(|| format!("failed to open file `{path}`", path = path.display()))
        .map_err(|e| function_call_failed("read_lines", format!("{e:?}"), context.call_site))?;

    let elements = BufReader::new(file)
        .lines()
        .map(|line| {
            Ok(PrimitiveValue::new_string(
                line.with_context(|| {
                    format!("failed to read file `{path}`", path = path.display())
                })
                .map_err(|e| {
                    function_call_failed("read_lines", format!("{e:?}"), context.call_site)
                })?,
            )
            .into())
        })
        .collect::<Result<Vec<Value>, _>>()?;

    Ok(Array::new_unchecked(context.return_type, Arc::new(elements)).into())
}

/// Gets the function describing `read_lines`.
pub const fn descriptor() -> Function {
    Function::new(const { &[Signature::new("(File) -> Array[String]", read_lines)] })
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use wdl_ast::version::V1;

    use crate::PrimitiveValue;
    use crate::v1::test::TestEnv;
    use crate::v1::test::eval_v1_expr;

    #[test]
    fn read_lines() {
        let mut env = TestEnv::default();
        env.write_file("foo", "\nhello!\nworld!\n\r\nhi!\r\nthere!");
        env.write_file("empty", "");
        env.insert_name("file", PrimitiveValue::new_file("foo"));

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "read_lines('does-not-exist')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .starts_with("call to function `read_lines` failed: failed to open file")
        );

        let value = eval_v1_expr(&mut env, V1::Two, "read_lines('foo')").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_string().unwrap().as_str())
            .collect();
        assert_eq!(elements, ["", "hello!", "world!", "", "hi!", "there!"]);

        let value = eval_v1_expr(&mut env, V1::Two, "read_lines(file)").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_string().unwrap().as_str())
            .collect();
        assert_eq!(elements, ["", "hello!", "world!", "", "hi!", "there!"]);

        let value = eval_v1_expr(&mut env, V1::Two, "read_lines('empty')").unwrap();
        assert!(value.unwrap_array().is_empty());
    }
}