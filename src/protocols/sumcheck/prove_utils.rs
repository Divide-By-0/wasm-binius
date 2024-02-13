// Copyright 2023 Ulvetanna Inc.

use rayon::{iter::Fold, prelude::*, range::Iter};
use std::borrow::Borrow;

use crate::{
	field::Field,
	polynomial::{
		extrapolate_line, multilinear_query::MultilinearQuery, CompositionPoly, EvaluationDomain,
		MultilinearComposite, MultilinearPoly,
	},
	protocols::sumcheck::{Error, SumcheckProof, SumcheckRound, SumcheckRoundClaim},
};

#[derive(Clone)]
pub struct PreSwitchoverWitness<F, M, BM>
where
	F: Field,
	M: MultilinearPoly<F> + ?Sized,
	BM: Borrow<M>,
{
	pub polynomial: MultilinearComposite<F, M, BM>,
	pub tensor: MultilinearQuery<F>,
}

#[derive(Clone)]
pub struct PreSwitchoverRoundOutput<F, M, BM>
where
	F: Field,
	M: MultilinearPoly<F> + ?Sized,
	BM: Borrow<M>,
{
	pub claim: SumcheckRoundClaim<F>,
	pub current_proof: SumcheckProof<F>,
	pub current_witness: PreSwitchoverWitness<F, M, BM>,
}

pub type PostSwitchoverWitness<F, M, BM> = MultilinearComposite<F, M, BM>;

#[derive(Clone)]
pub struct PostSwitchoverRoundOutput<F, OF, M, BM>
where
	F: Field,
	OF: Field,
	M: MultilinearPoly<OF> + ?Sized,
	BM: Borrow<M>,
{
	pub claim: SumcheckRoundClaim<F>,
	pub current_proof: SumcheckProof<F>,
	pub current_witness: PostSwitchoverWitness<OF, M, BM>,
}

#[inline]
fn process_round_evals<F: Field, C: CompositionPoly<F> + ?Sized>(
	composition: &C,
	evals_0: &[F],
	evals_1: &[F],
	evals_z: &mut [F],
	round_evals: &mut [F],
	domain: &[F],
) {
	let degree = domain.len() - 1;

	round_evals[0] += composition
		.evaluate_packed(evals_1)
		.expect("evals_1 is initialized with a length of poly.composition.n_vars()");

	for d in 2..degree + 1 {
		evals_0
			.iter()
			.zip(evals_1.iter())
			.zip(evals_z.iter_mut())
			.for_each(|((&evals_0_j, &evals_1_j), evals_z_j)| {
				*evals_z_j = extrapolate_line(evals_0_j, evals_1_j, domain[d]);
			});
		round_evals[d - 1] += composition
			.evaluate_packed(evals_z)
			.expect("evals_z is initialized with a length of poly.composition.n_vars()");
	}
}

fn calculate_round_evals_from_fold_result<F: Field>(
	degree: usize,
	fold_result: Fold<
		Iter<usize>,
		impl Fn() -> (Vec<F>, Vec<F>, Vec<F>, Vec<F>) + Send + Sync,
		impl Fn((Vec<F>, Vec<F>, Vec<F>, Vec<F>), usize) -> (Vec<F>, Vec<F>, Vec<F>, Vec<F>)
			+ Send
			+ Sync,
	>,
) -> Vec<F> {
	fold_result
		.map(|(_, _, _, round_evals)| round_evals)
		.reduce(
			|| vec![F::ZERO; degree],
			|mut overall_round_evals, partial_round_evals| {
				overall_round_evals
					.iter_mut()
					.zip(partial_round_evals.iter())
					.for_each(|(f, s)| *f += s);
				overall_round_evals
			},
		)
}

// Called for round 0 only
pub fn compute_round_coeffs_first<F, M, BM>(
	round_claim: SumcheckRoundClaim<F>,
	current_witness: PreSwitchoverWitness<F, M, BM>,
	domain: &EvaluationDomain<F>,
) -> Result<PreSwitchoverRoundOutput<F, M, BM>, Error>
where
	F: Field,
	M: MultilinearPoly<F> + Send + Sync + ?Sized,
	BM: Borrow<M> + Sync,
{
	let poly = current_witness.polynomial;
	let degree = poly.degree();
	let domain = domain.points();

	let n_multilinears = poly.composition.n_vars();
	let rd_vars = poly.n_vars();

	let fold_result = (0..1 << (rd_vars - 1)).into_par_iter().fold(
		|| {
			(
				vec![F::ZERO; n_multilinears],
				vec![F::ZERO; n_multilinears],
				vec![F::ZERO; n_multilinears],
				vec![F::ZERO; degree],
			)
		},
		|(mut evals_0, mut evals_1, mut evals_z, mut round_evals), i| {
			for (j, multilin) in poly.iter_multilinear_polys().enumerate() {
				evals_0[j] = multilin
					.evaluate_on_hypercube(i << 1)
					.unwrap_or_else(|_| panic!("tried to evaluate on hypercube vertex {}, but multilin has n_vars = {}, rd_vars is {}", 1 << i, multilin.n_vars(), rd_vars));
				evals_1[j] = multilin
					.evaluate_on_hypercube((i << 1) + 1)
    				.unwrap_or_else(|_| panic!("tried to evaluate on hypercube vertex {}, but multilin has n_vars = {}, rd_vars is {}", (1 << i) + 1, multilin.n_vars(), rd_vars));
			}
			process_round_evals(
				poly.composition.as_ref(),
				&evals_0,
				&evals_1,
				&mut evals_z,
				&mut round_evals,
				domain,
			);
			(evals_0, evals_1, evals_z, round_evals)
		},
	);

	let round_evals = calculate_round_evals_from_fold_result(degree, fold_result);

	let coeffs = round_evals;
	let current_proof = SumcheckProof {
		rounds: vec![SumcheckRound { coeffs }],
	};

	let result = PreSwitchoverRoundOutput {
		claim: round_claim,
		current_proof,
		current_witness: PreSwitchoverWitness {
			polynomial: poly,
			tensor: current_witness.tensor,
		},
	};
	Ok(result)
}

// Called for rounds 1 through s - 1 where s is the last round before the switchover
pub fn compute_round_coeffs_pre_switchover<F, M, BM>(
	updated_claim: SumcheckRoundClaim<F>,
	current_proof: SumcheckProof<F>,
	current_witness: PreSwitchoverWitness<F, M, BM>,
	domain: &EvaluationDomain<F>,
) -> Result<PreSwitchoverRoundOutput<F, M, BM>, Error>
where
	F: Field,
	M: MultilinearPoly<F> + Send + Sync + ?Sized,
	BM: Borrow<M> + Sync,
{
	let tensor = current_witness.tensor;
	let poly = current_witness.polynomial;
	let degree = poly.degree();
	let domain = domain.points();

	let mut updated_proof = current_proof;

	let n_multilinears = poly.n_multilinears();
	let rd_vars = poly.n_vars() - tensor.n_vars();

	let fold_result = (0..1 << (rd_vars - 1)).into_par_iter().fold(
		|| {
			(
				vec![F::ZERO; n_multilinears],
				vec![F::ZERO; n_multilinears],
				vec![F::ZERO; n_multilinears],
				vec![F::ZERO; degree],
			)
		},
		|(mut evals_0, mut evals_1, mut evals_z, mut round_evals), i| {
			for (j, multilin) in poly.iter_multilinear_polys().enumerate() {
				evals_0[j] = multilin
					.evaluate_subcube(i << 1, &tensor)
					.expect("index is in the range 0..(1 << rd_vars)");
				evals_1[j] = multilin
					.evaluate_subcube((i << 1) + 1, &tensor)
					.expect("index is in the range 0..(1 << rd_vars)");
			}
			process_round_evals(
				poly.composition.as_ref(),
				&evals_0,
				&evals_1,
				&mut evals_z,
				&mut round_evals,
				domain,
			);
			(evals_0, evals_1, evals_z, round_evals)
		},
	);

	let round_evals = calculate_round_evals_from_fold_result(degree, fold_result);

	// round_evals and round_claim, if honest, gives verifier enough information to
	// determine r(X) := \sum_{i \in B_{n-1}} poly(X, i)
	let coeffs = round_evals;
	updated_proof.rounds.push(SumcheckRound { coeffs });
	let result = PreSwitchoverRoundOutput {
		claim: updated_claim,
		current_proof: updated_proof,
		current_witness: PreSwitchoverWitness {
			polynomial: poly,
			tensor,
		},
	};
	Ok(result)
}

pub fn compute_round_coeffs_post_switchover<F, OF, M, BM>(
	updated_claim: SumcheckRoundClaim<F>,
	current_proof: SumcheckProof<F>,
	current_witness: PostSwitchoverWitness<OF, M, BM>,
	domain: &EvaluationDomain<F>,
) -> Result<PostSwitchoverRoundOutput<F, OF, M, BM>, Error>
where
	F: Field,
	OF: Field + Into<F> + From<F>,
	M: MultilinearPoly<OF> + Sync + ?Sized,
	BM: Borrow<M> + Sync,
{
	let poly = current_witness;
	let degree = poly.degree();
	let operating_domain = domain
		.points()
		.iter()
		.cloned()
		.map(OF::from)
		.collect::<Vec<_>>();

	let mut updated_proof = current_proof;

	let n_multilinears = poly.composition.n_vars();
	let rd_vars = poly.n_vars();

	let fold_result = (0..1 << (rd_vars - 1)).into_par_iter().fold(
		|| {
			(
				vec![OF::ZERO; n_multilinears],
				vec![OF::ZERO; n_multilinears],
				vec![OF::ZERO; n_multilinears],
				vec![OF::ZERO; degree],
			)
		},
		|(mut evals_0, mut evals_1, mut evals_z, mut round_evals), i| {
			for (j, multilin) in poly.iter_multilinear_polys().enumerate() {
				evals_0[j] = multilin.evaluate_on_hypercube(i << 1)
				.unwrap_or_else(|_| panic!("tried to evaluate on hypercube vertex {}, but multilin has n_vars = {}, rd_vars is {}", 1 << i, multilin.n_vars(), rd_vars));

				evals_1[j] = multilin.evaluate_on_hypercube((i << 1) + 1)
				.unwrap_or_else(|_| panic!("tried to evaluate on hypercube vertex {}, but multilin has n_vars = {}, rd_vars is {}", 1 << i, multilin.n_vars(), rd_vars));

			}
			process_round_evals(
				poly.composition.as_ref(),
				&evals_0,
				&evals_1,
				&mut evals_z,
				&mut round_evals,
				&operating_domain,
			);
			(evals_0, evals_1, evals_z, round_evals)
		},
	);

	let round_evals = calculate_round_evals_from_fold_result(degree, fold_result);

	let coeffs = round_evals
		.iter()
		.map(|&elem| elem.into())
		.collect::<Vec<_>>();

	updated_proof.rounds.push(SumcheckRound { coeffs });
	let result = PostSwitchoverRoundOutput {
		claim: updated_claim,
		current_proof: updated_proof,
		current_witness: poly,
	};
	Ok(result)
}