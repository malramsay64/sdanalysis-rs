//
// knn.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

//! Implement a K-Nearest Neighbours classification algorithm

use crate::learning::Classification;
use failure::{err_msg, Error};
use itertools::izip;
use rstar::{Point, PointDistance, RTree, RTreeObject, AABB};
use serde::{Deserialize, Serialize};

type Float = f32;

#[derive(Debug, Serialize, Deserialize)]
pub struct Features<F, L>
where
    F: Point<Scalar = Float>,
    L: Classification,
{
    features: F,
    label: L,
}

impl<F, L> Features<F, L>
where
    F: Point<Scalar = Float>,
    L: Classification,
{
    pub fn new(features: F, label: L) -> Features<F, L> {
        Features { label, features }
    }
}

impl<F, L> RTreeObject for Features<F, L>
where
    F: Point<Scalar = Float>,
    L: Classification,
{
    type Envelope = AABB<F>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.features)
    }
}

impl<F, L> PointDistance for Features<F, L>
where
    F: Point<Scalar = Float>,
    L: Classification,
{
    fn distance_2(&self, point: &F) -> F::Scalar {
        let mut distance = 0.;
        for i in 0..F::DIMENSIONS {
            let d = self.features.nth(i) - point.nth(i);
            distance += d * d;
        }
        distance
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KNN<F, L>
where
    F: Point<Scalar = Float>,
    L: Classification,
{
    tree: Option<RTree<Features<F, L>>>,
    k: usize,
}

impl<F, L> Default for KNN<F, L>
where
    F: Point<Scalar = Float>,
    L: Classification,
{
    fn default() -> KNN<F, L> {
        Self { tree: None, k: 5 }
    }
}

impl<F, L> KNN<F, L>
where
    F: Point<Scalar = Float>,
    L: Classification,
{
    /// Create an algorithm to classify new features into one of the labels
    ///
    /// Every time this function is run a new algorithm is generated, rather than updating or
    /// adding points to the existing one.
    ///
    pub fn fit(&mut self, features: &[F], labels: &[L]) {
        let values: Vec<Features<F, L>> = izip!(features, labels)
            .map(|(&feat, &class)| Features::new(feat, class))
            .collect();

        self.tree = Some(RTree::bulk_load(values));
    }

    pub fn predict(&self, features: &[F]) -> Result<Vec<L>, Error> {
        if let Some(tree) = &self.tree {
            // Find the k-Nearest Neighbours
            Ok(features
                .iter()
                .map(|feat| {
                    let values: Vec<L> = tree
                        .nearest_neighbor_iter(feat)
                        .take(self.k)
                        .map(|x| x.label)
                        .collect();

                    L::consensus(&values)
                })
                .collect())
        } else {
            Err(err_msg("The tree has not yet been initialised"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::Classes;

    #[test]
    fn simple_classification() -> Result<(), Error> {
        let mut knn = KNN::default();
        knn.fit(&vec![[0.; 2]; 10], &vec![Classes::Liquid; 10]);
        assert_eq!(knn.predict(&vec![[0.; 2]; 5])?, [Classes::Liquid; 5]);
        Ok(())
    }

    #[test]
    fn dual_classification() -> Result<(), Error> {
        let mut knn = KNN::default();
        let mut features = vec![[0.; 2]; 10];
        features.extend(&vec![[1.; 2]; 10]);
        let mut classes = vec![Classes::Liquid; 10];
        classes.extend(&vec![Classes::P2; 10]);
        knn.fit(&features, &classes);
        assert_eq!(knn.predict(&vec![[0.; 2]; 5])?, [Classes::Liquid; 5]);
        assert_eq!(knn.predict(&vec![[1.; 2]; 5])?, [Classes::P2; 5]);
        Ok(())
    }

    #[test]
    fn messy_classification() -> Result<(), Error> {
        let mut knn = KNN::default();
        let mut features = vec![[0.; 2]; 10];
        features.extend(&vec![[1.; 2]; 10]);
        features[0] = [1., 1.];
        features[10] = [0., 0.];
        let mut classes = vec![Classes::Liquid; 10];
        classes.extend(&vec![Classes::P2; 10]);
        knn.fit(&features, &classes);
        assert_eq!(knn.predict(&vec![[0.; 2]; 5])?, [Classes::Liquid; 5]);
        assert_eq!(knn.predict(&vec![[1.; 2]; 5])?, [Classes::P2; 5]);
        Ok(())
    }
}
