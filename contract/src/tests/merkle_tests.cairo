// Test helper functions
fn board_a() -> Array<u8> {
    array![
        3, 3, 3, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ]
}

fn salt_a() -> felt252 {
    9378218405727219894
}

fn board_b() -> Array<u8> {
    array![
        3, 3, 3, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ]
}

fn salt_b() -> felt252 {
    6894822432938596103
}

#[test]
fn test_commit_reveal() {
    let root_a = merkle::compute_merkle_root(board_a(), salt_a());
    println!("Root A {:x}", root_a);

    let root_b = merkle::compute_merkle_root(board_b(), salt_b());
    println!("Root B {:x}", root_b);
}
