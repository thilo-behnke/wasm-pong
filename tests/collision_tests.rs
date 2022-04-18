use rstest::rstest;
use rust_wasm::GameObject;
use rust_wasm::Collision;
use rust_wasm::CollisionDetector;

#[rstest]
#[case(vec![], vec![])]
pub fn should_detect_collisions(#[case] objs: Vec<&GameObject>, #[case] expected_collisions: Vec<Collision>) {
    let detector = CollisionDetector::new();
    let res = detector.detect_collisions(objs);
    assert_eq!(res, expected_collisions);
}
