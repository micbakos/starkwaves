pub fn compute_merkle_root(board: Array<u8>, salt: felt252) -> felt252 {
    let mut leaves: Array<felt252> = array![];
    let board_span = board.span();
    let mut i = 0;
    while i < board_span.len() {
        let cell: felt252 = (*board_span.at(i)).into();
        let hash = core::pedersen::pedersen(cell, salt);
        leaves.append(hash);
        i += 1;
    }

    compute_merkle_root_recursive(leaves.span())
}

fn compute_merkle_root_recursive(leaves: Span<felt252>) -> felt252 {
    let len = leaves.len();

    // Base cases
    if len == 0 {
        return 0;
    }

    if len == 1 {
        return *leaves.at(0);
    }

    let mut parent_level: Array<felt252> = array![];
    let mut i = 0;

    while i < len {
        if i + 1 < len {
            let left = *leaves.at(i);
            let right = *leaves.at(i + 1);
            let hash = core::pedersen::pedersen(left, right);
            parent_level.append(hash);
            i += 2;
        } else {
            // Odd node: duplicate it to create a pair
            let node = *leaves.at(i);
            let hash = core::pedersen::pedersen(node, node);
            parent_level.append(hash);
            i += 1;
        }
    }

    // Recurse on parent level
    compute_merkle_root_recursive(parent_level.span())
}

pub fn generate_proof(board: Array<u8>, salt: felt252, leaf_index: u32) -> Array<felt252> {
    let mut leaves: Array<felt252> = array![];
    let board_span = board.span();
    let mut i = 0;
    while i < board_span.len() {
        let cell: felt252 = (*board_span.at(i)).into();
        let hash = core::pedersen::pedersen(cell, salt);
        leaves.append(hash);
        i += 1;
    }

    generate_proof_recursive(leaves.span(), leaf_index)
}

fn generate_proof_recursive(leaves: Span<felt252>, mut index: u32) -> Array<felt252> {
    let len = leaves.len();

    if len <= 1 {
        return array![];
    }

    let mut proof: Array<felt252> = array![];
    let mut parent_level: Array<felt252> = array![];
    let mut parent_index: u32 = 0;
    let mut i: u32 = 0;

    while i < len {
        if i + 1 < len {
            let left = *leaves.at(i);
            let right = *leaves.at(i + 1);

            // If index is left child, add right sibling to proof
            if i == index {
                proof.append(right);
                parent_index = i / 2;
            } // If index is right child, add left sibling to proof
            else if i + 1 == index {
                proof.append(left);
                parent_index = i / 2;
            }

            let hash = core::pedersen::pedersen(left, right);
            parent_level.append(hash);
            i += 2;
        } else {
            // Odd node: duplicate it (hash with itself) and add as sibling
            let node = *leaves.at(i);
            if i == index {
                // Index is the odd node, add itself as sibling
                proof.append(node);
                parent_index = i / 2;
            }
            let hash = core::pedersen::pedersen(node, node);
            parent_level.append(hash);
            i += 1;
        }
    }

    let parent_proof = generate_proof_recursive(parent_level.span(), parent_index);
    let mut combined_proof: Array<felt252> = proof;
    let mut j = 0;
    while j < parent_proof.len() {
        combined_proof.append(*parent_proof.at(j));
        j += 1;
    }

    combined_proof
}

pub fn verify(
    salted_value: felt252, proof: Array<felt252>, root: felt252, leaf_index: usize,
) -> bool {
    let mut current_hash = salted_value;
    let mut current_index = leaf_index;
    let mut i = 0;

    while i < proof.len() {
        let sibling = *proof.at(i);

        if current_index % 2 == 0 {
            current_hash = core::pedersen::pedersen(current_hash, sibling);
        } else {
            current_hash = core::pedersen::pedersen(sibling, current_hash);
        }

        current_index = current_index / 2;
        i += 1;
    }

    current_hash == root
}
