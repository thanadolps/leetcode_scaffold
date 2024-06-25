use crate::api;
use crate::parse::{
    extract_code_snippet, parse_examples, parse_function_signature, Example, FuncInfo,
};
use color_eyre::{eyre::OptionExt, Result};
use std::fmt::Write;

pub(crate) fn generate_code(question: &api::Question) -> Result<String> {
    let snip = extract_code_snippet(
        question
            .codeSnippets
            .as_ref()
            .ok_or_eyre("expect code snippet")?,
        "rust",
    )
    .ok_or_eyre("expect rust's code snippet")?;
    let signature = parse_function_signature(&snip)?;

    let content = question
        .content
        .as_ref()
        .ok_or_eyre("expect question's content")?;
    let examples = parse_examples(content)?;

    let test = generate_test(&signature, &examples);
    Ok(format!("struct Solution;\n\n{snip}\n{test}"))
}

fn generate_test(signature: &FuncInfo, examples: &[Example]) -> String {
    // Generate test case (assume test_case crate is used)
    let test_cases: Vec<String> = examples.iter().map(generate_testcase).collect();
    let test_cases = test_cases.join("\n");

    // Generate test function
    // params into test function
    let mut params: Vec<String> = signature
        .inputs
        .iter()
        .map(|input| format!("{}: {}", input.name, input.type_))
        .collect();
    params.push(format!("expected: {}", signature.output_type));
    let params = params.join(", ");

    // args for calling the solution
    let input_args = signature
        .inputs
        .iter()
        .map(|input| input.name)
        .collect::<Vec<_>>()
        .join(", ");

    // Combine into test module
    format!(
        "
#[cfg(test)]
mod tests {{
    use super::*;
    use test_case::test_case;

{}
    fn test_{}({}) {{
        let res = Solution::{}({});
        assert_eq!(res, expected);
    }}
}}",
        test_cases, signature.name, params, signature.name, input_args
    )
}

fn generate_testcase(example: &Example) -> String {
    let mut items = Vec::new();
    items.extend(
        example
            .inputs
            .iter()
            .map(|(_, val)| val.replace('[', "vec![")),
    );
    items.push(example.output.to_owned());
    let items = items.join(", ");

    let name = example.name;

    let mut out = String::new();
    if let Some(explanation) = example.explanation {
        writeln!(out, "    // {explanation}").expect("write to string never fail");
    }
    write!(out, "    #[test_case({items}; {name:?})]").expect("write to string never fail");
    out
}
