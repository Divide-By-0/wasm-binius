// Copyright 2024 Ulvetanna Inc.

use crate::{
	polynomial::{
		Error as PolynomialError, MultilinearExtensionSpecialized, MultilinearPoly,
		MultilinearQuery,
	},
	protocols::sumcheck::Error,
};
use binius_field::{Field, PackedField};
use either::Either;
use rayon::prelude::*;
use std::{borrow::Borrow, cmp};

/// An individual multilinear polynomial in a multivariate composite.
#[derive(Debug)]
enum SumcheckMultilinear<P: PackedField, M> {
	/// Small field polynomial - to be folded into large field at `switchover` round
	Transparent {
		switchover: usize,
		small_field_multilin: M,
	},
	/// Large field polynomial - halved in size each round
	Folded {
		large_field_folded_multilin: MultilinearExtensionSpecialized<'static, P, P>,
	},
}

/// Parallel fold state, consisting of scratch area and result accumulator.
struct ParFoldState<F: Field> {
	// Evaluations at 0, 1 and domain points, per MLE. Scratch space.
	evals_0: Vec<F>,
	evals_1: Vec<F>,
	evals_z: Vec<F>,

	// Accumulated sums of evaluations over univariate domain.
	round_evals: Vec<F>,
}

impl<F: Field> ParFoldState<F> {
	fn new(n_multilinears: usize, n_round_evals: usize) -> Self {
		Self {
			evals_0: vec![F::ZERO; n_multilinears],
			evals_1: vec![F::ZERO; n_multilinears],
			evals_z: vec![F::ZERO; n_multilinears],
			round_evals: vec![F::ZERO; n_round_evals],
		}
	}
}

/// Represents an object that can evaluate the composition function of a generalized sumcheck.
///
/// Generalizes handling of regular sumcheck and zerocheck protocols.
pub trait SumcheckEvaluator<F>: Send + Sync {
	/// The number of points to evaluate at.
	fn n_round_evals(&self) -> usize;

	/// Process and update the round evaluations with the evaluations at a hypercube vertex.
	///
	/// ## Arguments
	///
	/// * `index`: index of the hypercube vertex
	/// * `evals_0`: the n multilinear polynomial evaluations at 0
	/// * `evals_1`: the n multilinear polynomial evaluations at 1
	/// * `evals_z`: a scratch buffer of size n for storing multilinear polynomial evaluations at a
	///              point z
	/// * `round_evals`: the accumulated evaluations for the round
	fn process_vertex(
		&self,
		index: usize,
		evals_0: &[F],
		evals_1: &[F],
		evals_z: &mut [F],
		round_evals: &mut [F],
	);
}

impl<F, L, R> SumcheckEvaluator<F> for Either<L, R>
where
	L: SumcheckEvaluator<F>,
	R: SumcheckEvaluator<F>,
{
	fn n_round_evals(&self) -> usize {
		match self {
			Either::Left(left) => left.n_round_evals(),
			Either::Right(right) => right.n_round_evals(),
		}
	}

	fn process_vertex(
		&self,
		index: usize,
		evals_0: &[F],
		evals_1: &[F],
		evals_z: &mut [F],
		round_evals: &mut [F],
	) {
		match self {
			Either::Left(left) => {
				left.process_vertex(index, evals_0, evals_1, evals_z, round_evals)
			}
			Either::Right(right) => {
				right.process_vertex(index, evals_0, evals_1, evals_z, round_evals)
			}
		}
	}
}

/// A prover state for a generalized sumcheck protocol.
///
/// The family of generalized sumcheck protocols includes regular sumcheck and zerocheck. The
/// zerocheck case permits many important optimizations, enumerated in [Gruen24]. These algorithms
/// are used to prove the interactive multivariate sumcheck protocol in the specific case that the
/// polynomial is a composite of multilinears. This prover state is responsible for updating and
/// evaluating the composed multilinears.
///
/// Once initialized, the expected caller behavior is to alternate invocations of
/// [`Self::sum_round_evals`] and [`Self::fold`], for a total of `n_rounds` calls to each.
///
/// We associate with each multilinear a `switchover` round number, which controls small field
/// optimization and corresponding time/memory tradeoff. In rounds $0, \ldots, switchover-1$ the
/// partial evaluation of a specific multilinear is obtained by doing $2^{n\\_vars - round}$ inner
/// products, with total time complexity proportional to the number of polynomial coefficients.
///
/// After switchover the inner products are stored in a new MLE in large field, which is halved on
/// each round. There are two tradeoffs at play:
///   1) Pre-switchover rounds perform Small * Large field multiplications, but do $2^{round}$ as many of them.
///   2) Pre-switchover rounds require no additional memory, but initial folding allocates a new MLE in a
///      large field that is $2^{switchover}$ times smaller - for example for 1-bit polynomial and 128-bit large
///      field a switchover of 7 would require additional memory identical to the polynomial size.
///
/// NB. Note that `switchover=0` does not make sense, as first round is never folded.//
///
/// [Gruen24]: https://eprint.iacr.org/2024/108
#[derive(Debug)]
pub struct ProverState<PW, M>
where
	PW: PackedField,
	M: MultilinearPoly<PW> + Sync,
{
	multilinears: Vec<SumcheckMultilinear<PW, M>>,
	query: Option<MultilinearQuery<PW>>,
	round: usize,
	n_rounds: usize,
}

impl<PW, M> ProverState<PW, M>
where
	PW: PackedField,
	M: MultilinearPoly<PW> + Sync,
{
	pub fn new(
		n_rounds: usize,
		multilinears: impl IntoIterator<Item = M>,
		switchover_fn: impl Fn(usize) -> usize,
	) -> Result<Self, Error> {
		let mut max_query_vars = 1;
		let multilinears = multilinears
			.into_iter()
			.map(|small_field_multilin| {
				if small_field_multilin.n_vars() != n_rounds {
					return Err(PolynomialError::IncorrectNumberOfVariables {
						expected: n_rounds,
						actual: small_field_multilin.n_vars(),
					}
					.into());
				}

				let switchover = switchover_fn(small_field_multilin.extension_degree());
				max_query_vars = cmp::max(max_query_vars, switchover);
				Ok(SumcheckMultilinear::Transparent {
					switchover,
					small_field_multilin,
				})
			})
			.collect::<Result<_, Error>>()?;

		let query = Some(MultilinearQuery::new(max_query_vars)?);

		Ok(Self {
			multilinears,
			query,
			round: 0,
			n_rounds,
		})
	}

	/// Fold all stored multilinears with the verifier challenge received in the previous round.
	///
	/// This manages whether to partially evaluate the multilinear at an extension point
	/// (post-switchover) or to store the extended tensor product of the received queries
	/// (pre-switchover).
	///
	/// See struct documentation for more details on the generalized sumcheck proving algorithm.
	pub fn fold(&mut self, prev_rd_challenge: PW::Scalar) -> Result<(), Error> {
		let &mut Self {
			ref mut multilinears,
			ref mut query,
			ref mut round,
			..
		} = self;

		*round += 1;

		// Update query (has to be done before switchover)
		if let Some(prev_query) = query.take() {
			let expanded_query = prev_query.update(&[prev_rd_challenge])?;
			query.replace(expanded_query);
		}

		// Partial query (for folding)
		let partial_query = MultilinearQuery::with_full_query(&[prev_rd_challenge])?;

		// Perform switchover and/or folding
		let mut any_transparent_left = false;

		for multilin in multilinears.iter_mut() {
			match *multilin {
				SumcheckMultilinear::Transparent {
					switchover,
					ref small_field_multilin,
				} => {
					if switchover <= *round {
						let query_ref = query.as_ref().expect(
							"query is guaranteed to be Some while there are transparent \
								multilinears remaining",
						);
						// At switchover, perform inner products in large field and save them
						// in a newly created MLE.
						let large_field_folded_multilin = small_field_multilin
							.borrow()
							.evaluate_partial_low(query_ref)?;

						*multilin = SumcheckMultilinear::Folded {
							large_field_folded_multilin,
						};
					} else {
						any_transparent_left = true;
					}
				}

				SumcheckMultilinear::Folded {
					ref mut large_field_folded_multilin,
				} => {
					// Post-switchover, simply halve large field MLE.
					*large_field_folded_multilin =
						large_field_folded_multilin.evaluate_partial_low(&partial_query)?;
				}
			}
		}

		// All folded large field - tensor is no more needed.
		if !any_transparent_left {
			*query = None;
		}

		Ok(())
	}

	/// Compute the sum of the partial polynomial evaluations over the hypercube.
	pub fn sum_round_evals(
		&self,
		evaluator: impl SumcheckEvaluator<PW::Scalar>,
	) -> Vec<PW::Scalar> {
		// Extract multilinears & round
		let &Self {
			ref multilinears,
			round,
			..
		} = self;

		// Handling different cases separately for more inlining opportunities
		// (especially in early rounds)
		let any_transparent = multilinears
			.iter()
			.any(|ml| matches!(ml, SumcheckMultilinear::Transparent { .. }));
		let any_folded = multilinears
			.iter()
			.any(|ml| matches!(ml, SumcheckMultilinear::Folded { .. }));

		match (any_transparent, any_folded) {
			(true, false) => {
				if round == 0 {
					// All transparent, first round - direct sampling
					self.sum_round_evals_helper(
						Self::only_transparent,
						Self::direct_sample,
						evaluator,
					)
				} else {
					// All transparent, rounds 1..n_vars - small field inner product
					self.sum_round_evals_helper(
						Self::only_transparent,
						|multilin, i| self.subcube_inner_product(multilin, i),
						evaluator,
					)
				}
			}

			// All folded - direct sampling
			(false, true) => {
				self.sum_round_evals_helper(Self::only_folded, Self::direct_sample, evaluator)
			}

			// Heterogeneous case
			_ => self.sum_round_evals_helper(
				|x| x,
				|sc_multilin, i| match sc_multilin {
					SumcheckMultilinear::Transparent {
						small_field_multilin,
						..
					} => self.subcube_inner_product(small_field_multilin.borrow(), i),

					SumcheckMultilinear::Folded {
						large_field_folded_multilin,
					} => Self::direct_sample(large_field_folded_multilin, i),
				},
				evaluator,
			),
		}
	}

	// The gist of sumcheck - summing over evaluations of the multivariate composite on evaluation domain
	// for the remaining variables: there are `round-1` already assigned variables with values from large
	// field, and `rd_vars = n_vars - round` remaining variables that are being summed over. `eval01` closure
	// computes 0 & 1 evaluations at some index - either by performing inner product over assigned variables
	// pre-switchover or directly sampling MLE representation during first round or post-switchover.
	fn sum_round_evals_helper<'b, T>(
		&'b self,
		precomp: impl Fn(&'b SumcheckMultilinear<PW, M>) -> T,
		eval01: impl Fn(T, usize) -> (PW::Scalar, PW::Scalar) + Sync,
		evaluator: impl SumcheckEvaluator<PW::Scalar>,
	) -> Vec<PW::Scalar>
	where
		T: Copy + Sync + 'b,
		M: 'b,
	{
		let rd_vars = self.n_rounds - self.round;
		let n_multilinears = self.multilinears.len();
		let n_round_evals = evaluator.n_round_evals();

		// When possible to pre-process unpacking sumcheck multilinears, we do so.
		// For performance, it's ideal to hoist this out of the tight loop.
		let precomps = self.multilinears.iter().map(precomp).collect::<Vec<_>>();

		(0..1 << (rd_vars - 1))
			.into_par_iter()
			.fold(
				|| ParFoldState::new(n_multilinears, n_round_evals),
				|mut state, i| {
					for (j, precomp) in precomps.iter().enumerate() {
						let (eval0, eval1) = eval01(*precomp, i);
						state.evals_0[j] = eval0;
						state.evals_1[j] = eval1;
					}

					evaluator.process_vertex(
						i,
						&state.evals_0,
						&state.evals_1,
						&mut state.evals_z,
						&mut state.round_evals,
					);

					state
				},
			)
			.map(|state| state.round_evals)
			// Simply sum up the fold partitions.
			.reduce(
				|| vec![PW::Scalar::ZERO; n_round_evals],
				|mut overall_round_evals, partial_round_evals| {
					overall_round_evals
						.iter_mut()
						.zip(partial_round_evals.iter())
						.for_each(|(f, s)| *f += s);
					overall_round_evals
				},
			)
	}

	// Note the generic parameter - this method samples small field in first round and
	// large field post-switchover.
	#[inline]
	fn direct_sample<MD>(multilin: MD, i: usize) -> (PW::Scalar, PW::Scalar)
	where
		MD: MultilinearPoly<PW>,
	{
		let eval0 = multilin
			.evaluate_on_hypercube(i << 1)
			.expect("eval 0 within range");
		let eval1 = multilin
			.evaluate_on_hypercube((i << 1) + 1)
			.expect("eval 1 within range");

		(eval0, eval1)
	}

	#[inline]
	fn subcube_inner_product(&self, multilin: &M, i: usize) -> (PW::Scalar, PW::Scalar) where {
		let query = self.query.as_ref().expect("tensor present by invariant");

		let eval0 = multilin
			.evaluate_subcube(i << 1, query)
			.expect("eval 0 within range");
		let eval1 = multilin
			.evaluate_subcube((i << 1) + 1, query)
			.expect("eval 1 within range");

		(eval0, eval1)
	}

	fn only_transparent(sc_multilin: &SumcheckMultilinear<PW, M>) -> &M {
		match sc_multilin {
			SumcheckMultilinear::Transparent {
				small_field_multilin,
				..
			} => small_field_multilin.borrow(),
			_ => panic!("all transparent by invariant"),
		}
	}

	fn only_folded(
		sc_multilin: &SumcheckMultilinear<PW, M>,
	) -> &MultilinearExtensionSpecialized<'static, PW, PW> {
		match sc_multilin {
			SumcheckMultilinear::Folded {
				large_field_folded_multilin,
			} => large_field_folded_multilin,
			_ => panic!("all folded by invariant"),
		}
	}
}
