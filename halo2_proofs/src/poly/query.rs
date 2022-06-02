use std::{fmt::Debug, ops::Deref};

use super::commitment::{Blind, CommitmentScheme, Params, MSM};
use crate::{
    arithmetic::eval_polynomial,
    poly::{commitment, Coeff, Polynomial},
};
use ff::Field;
use halo2curves::CurveAffine;

pub trait Query<F>: Sized + Clone {
    type Commitment: PartialEq + Copy;
    type Eval: Clone + Default + Debug;

    fn get_point(&self) -> F;
    fn get_eval(&self) -> Self::Eval;
    fn get_commitment(&self) -> Self::Commitment;
}

/// A polynomial query at a point
#[derive(Debug, Clone)]
pub struct ProverQuery<'com, C: CurveAffine> {
    /// point at which polynomial is queried
    pub(crate) point: C::Scalar,
    /// coefficients of polynomial
    pub(crate) poly: &'com Polynomial<C::Scalar, Coeff>,
    /// blinding factor of polynomial
    pub(crate) blind: Blind<C::Scalar>,
}

#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct PolynomialPointer<'com, C: CurveAffine> {
    pub(crate) poly: &'com Polynomial<C::Scalar, Coeff>,
    pub(crate) blind: Blind<C::Scalar>,
}

impl<'com, C: CurveAffine> PartialEq for PolynomialPointer<'com, C> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.poly, other.poly)
    }
}

impl<'com, C: CurveAffine> Query<C::Scalar> for ProverQuery<'com, C> {
    type Commitment = PolynomialPointer<'com, C>;
    type Eval = C::Scalar;

    fn get_point(&self) -> C::Scalar {
        self.point
    }
    fn get_eval(&self) -> Self::Eval {
        eval_polynomial(&self.poly[..], self.get_point())
    }
    fn get_commitment(&self) -> Self::Commitment {
        PolynomialPointer {
            poly: self.poly,
            blind: self.blind,
        }
    }
}

impl<'com, C: CurveAffine> VerifierQuery<'com, C> {
    /// Create a new verifier query based on a commitment
    pub fn new_commitment(commitment: &'com C, point: C::Scalar, eval: C::Scalar) -> Self {
        VerifierQuery {
            point,
            eval,
            commitment: CommitmentReference::Commitment(commitment),
        }
    }

    /// Create a new verifier query based on a linear combination of commitments
    pub fn new_msm(
        msm: &'com dyn MSM<C>,
        point: C::Scalar,
        eval: C::Scalar,
    ) -> VerifierQuery<'com, C> {
        VerifierQuery {
            point,
            eval,
            commitment: CommitmentReference::MSM(msm),
        }
    }
}

/// A polynomial query at a point
#[derive(Debug, Clone)]
pub struct VerifierQuery<'com, C: CurveAffine> {
    /// point at which polynomial is queried
    pub(crate) point: C::Scalar,
    /// commitment to polynomial
    pub(crate) commitment: CommitmentReference<'com, C>,
    /// evaluation of polynomial at query point
    pub(crate) eval: C::Scalar,
}

#[derive(Copy, Clone, Debug)]
pub enum CommitmentReference<'r, C: CurveAffine> {
    Commitment(&'r C),
    MSM(&'r dyn MSM<C>),
}

impl<'r, C: CurveAffine> PartialEq for CommitmentReference<'r, C> {
    #![allow(clippy::vtable_address_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&CommitmentReference::Commitment(a), &CommitmentReference::Commitment(b)) => {
                std::ptr::eq(a, b)
            }
            (&CommitmentReference::MSM(a), &CommitmentReference::MSM(b)) => std::ptr::eq(a, b),
            _ => false,
        }
    }
}

impl<'com, C: CurveAffine> Query<C::Scalar> for VerifierQuery<'com, C> {
    type Eval = C::Scalar;
    type Commitment = CommitmentReference<'com, C>;

    fn get_point(&self) -> C::Scalar {
        self.point
    }
    fn get_eval(&self) -> C::Scalar {
        self.eval
    }
    fn get_commitment(&self) -> Self::Commitment {
        self.commitment
    }
}