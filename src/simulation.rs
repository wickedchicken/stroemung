pub fn run_simulation() {
    todo!("Assign initial values to u, v, p");
    for _ in 0..1 {
        // while t < t_end
        todo!("Set boundary values for u and v");
        todo!("Compute F and G");
        todo!("Compute rhs of pressure equation");
        for _ in 0..100 {
            // while it < it_max, ||rit|| > eps
            todo!("Perform SOR cycle");
            todo!("Compute residual norm");
        }
        todo!("Compute u(n+1) and v(n+1)");
    }
}
