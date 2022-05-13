mod test {
    use std::collections::HashMap;
    use rstest::rstest;

    #[rstest]
    #[case(
    "test=abc",
    HashMap::from([("test", "abc")])
    )]
    fn get_query_params_tests(#[case] query_str: &str, #[case] expected: HashMap<&str, &str>) {
        let res = get_(query_str);
        assert_eq!(res, expected)
    }
}
