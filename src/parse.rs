use crate::api::CodeSnippetNode;
use color_eyre::{eyre::OptionExt, Result};
use regex::Regex;
use std::collections::BTreeMap;

pub(crate) fn extract_code_snippet(snippets: &[CodeSnippetNode], lang: &str) -> Option<String> {
    snippets
        .iter()
        .find(|snippet| snippet.langSlug == lang)
        .map(|snippet| snippet.code.clone())
}

#[derive(Debug, PartialEq)]
pub(crate) struct SnippetInput<'a> {
    pub(crate) name: &'a str,
    pub(crate) type_: &'a str,
}

#[derive(Debug, PartialEq)]
pub(crate) struct FuncInfo<'a> {
    pub(crate) name: &'a str,
    pub(crate) inputs: Vec<SnippetInput<'a>>,
    pub(crate) output_type: &'a str,
}

/// Parse information about function signature from code snippet
pub(crate) fn parse_function_signature(snippet: &str) -> Result<FuncInfo> {
    let re = Regex::new(r"fn\s+(\w+)\(([^)]+)\)\s*->\s*(\w+)").unwrap();

    let captures = re
        .captures(snippet)
        .ok_or_eyre("cannot extract function signature")?;
    let (_, [name, inputs, output_type]) = captures.extract();

    let inputs = inputs
        .split(',')
        .map(|input| {
            let (name, type_) = input
                .split_once(':')
                .ok_or_eyre("function's parameter should have annotated type")?;
            Ok(SnippetInput {
                name: name.trim(),
                type_: type_.trim(),
            })
        })
        .collect::<Result<_>>()?;

    Ok(FuncInfo {
        name,
        inputs,
        output_type,
    })
}

#[derive(Debug)]
pub(crate) struct Example<'a> {
    pub(crate) name: &'a str,
    pub(crate) inputs: Vec<(&'a str, &'a str)>,
    pub(crate) output: &'a str,
}

/// Extract and parse examples from leetcode problem's html content/description
pub(crate) fn parse_examples(content: &str) -> Result<Vec<Example>> {
    let header_re = Regex::new(r#"<p><strong class="example">(.+)<\/strong><\/p>"#).unwrap();
    let case_re =
        Regex::new(r"<strong>Input:<\/strong>(.+)\s*<strong>Output:<\/strong>(.+)").unwrap();

    let headers: Vec<&str> = header_re
        .captures_iter(content)
        .map(|cap| {
            let (_, [title]) = cap.extract();
            title.trim_end_matches(':')
        })
        .collect();

    let io: Vec<(&str, &str)> = case_re
        .captures_iter(content)
        .map(|cap| {
            let (_, [input, output]) = cap.extract();
            (input.trim(), output.trim())
        })
        .collect();

    assert_eq!(headers.len(), io.len());

    std::iter::zip(headers, io)
        .map(|(name, (input, output))| {
            let inputs = split_example_input(input.trim())
                .into_iter()
                .map(|i| {
                    i.split_once('=')
                        .ok_or_eyre("expected input token in form of \"`var` = `val`\"")
                        .map(|(var, val)| (var.trim(), val.trim()))
                })
                .collect::<Result<Vec<(&str, &str)>>>()?;

            Ok(Example {
                name,
                inputs,
                output: output.trim(),
            })
        })
        .collect()
}

/// Split the string in "example input" format into individual inputs.
///
/// The format is simply a comma separated string,
/// but care is taken to not split comma within expressions such as in array or string.
///
/// # Example
/// ```
/// let input = "position = [1,2,3,4,7], m = 3";
/// let inputs = split_example_input(input);
/// assert_eq!(inputs, ["position = [1,2,3,4,7]", "m = 3"]);
/// ```
fn split_example_input(input: &str) -> Vec<&str> {
    if input.is_empty() {
        return Vec::new();
    }

    let bracket_list: BTreeMap<char, char> = [
        ('(', ')'),
        ('[', ']'),
        ('{', '}'),
        ('<', '>'),
        ('\'', '\''),
        ('"', '"'),
    ]
    .into_iter()
    .collect();

    let mut inputs = Vec::new(); // store splitted inputs
    let mut start = 0; // start index of next splitted input
    let mut stack = Vec::new(); // stack to keep track of brackets
    for (i, c) in input.chars().enumerate() {
        // pop the stack when closing bracket is found
        if stack.last() == Some(&c) {
            stack.pop();
            continue;
        }

        // push matching bracket to stack when opening bracket is found
        if let Some(&bracket) = bracket_list.get(&c) {
            stack.push(bracket);
            continue;
        }

        // split at comma if it's not within brackets
        if c == ',' && stack.is_empty() {
            inputs.push(input[start..i].trim());
            start = i + 1;
        }
    }
    assert!(stack.is_empty(), "Unmatched brackets in input: {input}");
    inputs.push(input[start..].trim());
    inputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_code_snippet() {
        let snippet = r"
        fn max_distance(position: Vec<i32>, m: i32) -> i32 {
            0
        }
        ";

        let info = parse_function_signature(snippet).unwrap();
        assert_eq!(
            info,
            FuncInfo {
                name: "max_distance",
                inputs: vec![
                    SnippetInput {
                        name: "position",
                        type_: "Vec<i32>"
                    },
                    SnippetInput {
                        name: "m",
                        type_: "i32"
                    }
                ],
                output_type: "i32"
            }
        );
    }

    //     #[test]
    //     fn test_parse_examples() {
    //         let content = r#"
    //         <p>In the universe Earth C-137, Rick discovered a special form of magnetic force between two balls if they are put in his new invented basket. Rick has <code>n</code> empty baskets, the <code>i<sup>th</sup></code> basket is at <code>position[i]</code>, Morty has <code>m</code> balls and needs to distribute the balls into the baskets such that the <strong>minimum magnetic force</strong> between any two balls is <strong>maximum</strong>.</p>

    //         <p>Rick stated that magnetic force between two different balls at positions <code>x</code> and <code>y</code> is <code>|x - y|</code>.</p>

    //         <p>Given the integer array <code>position</code> and the integer <code>m</code>. Return <em>the required force</em>.</p>

    //         <p>&nbsp;</p>
    //         <p><strong class="example">Example 1:</strong></p>
    //         <img alt="" src="https://assets.leetcode.com/uploads/2020/08/11/q3v1.jpg" style="width: 562px; height: 195px;" />
    //         <pre>
    //         <strong>Input:</strong> position = [1,2,3,4,7], m = 3
    //         <strong>Output:</strong> 3
    //         <strong>Explanation:</strong> Distributing the 3 balls into baskets 1, 4 and 7 will make the magnetic force between ball pairs [3, 3, 6]. The minimum magnetic force is 3. We cannot achieve a larger minimum magnetic force than 3.
    //         </pre>

    //         <p><strong class="example">Example 2:</strong></p>

    //         <pre>
    //         <strong>Input:</strong> position = [5,4,3,2,1,1000000000], m = 2
    //         <strong>Output:</strong> 999999999
    //         <strong>Explanation:</strong> We can use baskets 1 and 1000000000.
    //         </pre>

    //         <p>&nbsp;</p>
    //         <p><strong>Constraints:</strong></p>

    //         <ul>
    // 	<li><code>n == position.length</code></li>
    // 	<li><code>2 &lt;= n &lt;= 10<sup>5</sup></code></li>
    // 	<li><code>1 &lt;= position[i] &lt;= 10<sup>9</sup></code></li>
    // 	<li>All integers in <code>position</code> are <strong>distinct</strong>.</li>
    // 	<li><code>2 &lt;= m &lt;= position.length</code></li>
    //         </ul>
    // "#;

    //         dbg!(parse_examples(content));
    //     }

    #[test]
    fn test_split_example_input() {
        assert_eq!(split_example_input(""), [""; 0]);
        assert_eq!(
            split_example_input(r#"a = "1,2", b=3"#),
            [r#"a = "1,2""#, "b=3"]
        );
        assert_eq!(
            split_example_input(r#"s = "catsanddog""#),
            [r#"s = "catsanddog""#]
        );
        assert_eq!(split_example_input("a = 123, c = d"), ["a = 123", "c = d"]);
        assert_eq!(
            split_example_input("a = String::new(1,2,3), b=5"),
            ["a = String::new(1,2,3)", "b=5"]
        );
        assert_eq!(
            split_example_input(r#"a = [1,2,3], b = 4, dict = [["hello"], ["word"]], c = 5"#),
            [
                "a = [1,2,3]",
                "b = 4",
                r#"dict = [["hello"], ["word"]]"#,
                "c = 5"
            ]
        );
    }
}
