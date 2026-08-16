fn public_oracle() {
    if add(2, 3) != 5 {
        panic!("repair failed");
    }
}
