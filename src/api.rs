//! This module contains functions for fetching question data from `LeetCode` API.
//!
//! It's a extract from [leetcoderustapi](https://lib.rs/crates/leetcoderustapi) with slight modifications.

use color_eyre::Result;
use serde::Deserialize;
use serde_json::json;

pub(crate) fn fetch_problem_full_data(problem: &str) -> Result<(String, Question)> {
    #[derive(serde_query::Deserialize)]
    struct Response {
        #[query(".data.question")]
        question: Question,
    }

    let query = json!({
        "operationName": "questionData",
        "variables": {
            "titleSlug": problem
        },
        "query": "query questionData($titleSlug: String!) {\n  question(titleSlug: $titleSlug) {\n    questionId\n    questionFrontendId\n    boundTopicId\n    title\n    titleSlug\n    content\n    translatedTitle\n    translatedContent\n    isPaidOnly\n    canSeeQuestion\n    difficulty\n    likes\n    dislikes\n    isLiked\n    similarQuestions\n    exampleTestcases\n    categoryTitle\n    contributors {\n      username\n      profileUrl\n      avatarUrl\n      __typename\n    }\n    topicTags {\n      name\n      slug\n      translatedName\n      __typename\n    }\n    companyTagStats\n    codeSnippets {\n      lang\n      langSlug\n      code\n      __typename\n    }\n    stats\n    hints\n    solution {\n      id\n      canSeeDetail\n      paidOnly\n      hasVideoSolution\n      paidOnlyVideo\n      __typename\n    }\n    status\n    sampleTestCase\n    metaData\n    judgerAvailable\n    judgeType\n    mysqlSchemas\n    enableRunCode\n    enableTestMode\n    enableDebugger\n    envInfo\n    libraryUrl\n    adminUrl\n    challengeQuestion {\n      id\n      date\n      incompleteChallengeCount\n      streakCount\n      type\n      __typename\n    }\n    __typename\n  }\n}"
    });

    let question = ureq::post("https://leetcode.com/graphql/")
        .send_json(query)?
        .into_json::<Response>()?
        .question;

    Ok((question.titleSlug.clone(), question))
}

pub(crate) fn get_question_name(keyword: &str) -> Result<String> {
    #[derive(serde_query::Deserialize)]
    struct Response {
        #[query(".data.problemsetQuestionList.questions.[0].titleSlug")]
        title: String,
    }

    let query = json!({
        "query": "query problemsetQuestionList($categorySlug: String, $limit: Int, $skip: Int, $filters: QuestionListFilterInput) { problemsetQuestionList: questionList( categorySlug: $categorySlug limit: $limit skip: $skip filters: $filters ) { questions: data { titleSlug } } }",
        "variables": {
            "categorySlug": "",
            "skip": 0,
            "limit": 1,
            "filters": {
                "searchKeywords": keyword
            }
        },
        "operationName": "problemsetQuestionList"
    });

    let title = ureq::post("https://leetcode.com/graphql/")
        .send_json(query)?
        .into_json::<Response>()?
        .title;
    Ok(title)
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Question {
    // pub(crate) questionId: String,
    pub(crate) questionFrontendId: String,
    // pub(crate) boundTopicId: Option<String>,
    // pub(crate) title: String,
    pub(crate) titleSlug: String,
    pub(crate) content: Option<String>,
    // pub(crate) translatedTitle: Option<String>,
    // pub(crate) translatedContent: Option<String>,
    // pub(crate) isPaidOnly: bool,
    // pub(crate) canSeeQuestion: bool,
    // pub(crate) difficulty: String,
    // pub(crate) likes: u32,
    // pub(crate) dislikes: u32,
    // pub(crate) isLiked: Option<bool>,
    // pub(crate) similarQuestions: String,
    // pub(crate) exampleTestcases: String,
    // pub(crate) categoryTitle: String,
    // pub(crate) contributors: Vec<String>,
    // pub(crate) topicTags: Vec<TopicTagNode>,
    // pub(crate) companyTagStats: Option<String>,
    pub(crate) codeSnippets: Option<Vec<CodeSnippetNode>>,
    // pub(crate) stats: String,
    // pub(crate) hints: Vec<String>,
    // pub(crate) solution: Option<Solution>,
    // pub(crate) status: Option<String>,
    // pub(crate) sampleTestCase: String,
    // pub(crate) metaData: String,
    // pub(crate) judgerAvailable: Option<bool>,
    // pub(crate) judgeType: Option<String>,
    // pub(crate) mysqlSchemas: Option<Vec<String>>,
    // pub(crate) enableRunCode: Option<bool>,
    // pub(crate) enableTestMode: Option<bool>,
    // pub(crate) enableDebugger: Option<bool>,
    // pub(crate) envInfo: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct CodeSnippetNode {
    // pub(crate) lang: String,
    pub(crate) langSlug: String,
    pub(crate) code: String,
}

// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize, Clone)]
// pub(crate) struct Solution {
//     pub(crate) id: String,
//     pub(crate) canSeeDetail: bool,
//     pub(crate) paidOnly: bool,
//     pub(crate) hasVideoSolution: bool,
//     pub(crate) paidOnlyVideo: bool,
// }

// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize, Clone)]
// pub(crate) struct TopicTagNode {
//     pub(crate) name: String,
//     pub(crate) slug: String,
//     pub(crate) translatedName: Option<String>,
// }
