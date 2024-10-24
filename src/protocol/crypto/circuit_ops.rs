use std::ops::Mul;

use crate::protocol::*;

pub trait CircuitMul<T>: Mul<T, Output = T> {
    fn circuit_mul(
        builder: &mut CircuitBuilder,
        lhs: HashOutTarget,
        rhs: HashOutTarget,
    ) -> HashOutTarget;
}
