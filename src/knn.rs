//
// knn.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

//! Implement a K-NN classification algorithm

use failure::{err_msg, Error};
use itertools::izip;
use rstar::{Point, PointDistance, RTree, RTreeObject, AABB};
use serde::{Deserialize, Serialize};

type Float = f32;

#[derive(Debug, Serialize, Deserialize)]
pub struct Features<T>
where
    T: Point<Scalar = Float>,
{
    label: usize,
    features: T,
}

impl<T> Features<T>
where
    T: Point<Scalar = Float>,
{
    pub fn new(features: T, label: usize) -> Features<T> {
        Features { label, features }
    }
}

impl<T> RTreeObject for Features<T>
where
    T: Point<Scalar = Float>,
{
    type Envelope = AABB<T>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.features)
    }
}

impl<T> PointDistance for Features<T>
where
    T: Point<Scalar = Float>,
{
    fn distance_2(&self, point: &T) -> T::Scalar {
        let mut distance = 0.;
        for i in 0..T::DIMENSIONS {
            let d = self.features.nth(i) - point.nth(i);
            distance += d * d;
        }
        distance
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KNN<T>
where
    T: Point<Scalar = Float>,
{
    tree: Option<RTree<Features<T>>>,
    k: usize,
}

impl<T> Default for KNN<T>
where
    T: Point<Scalar = Float>,
{
    fn default() -> KNN<T> {
        Self { tree: None, k: 5 }
    }
}

impl<T: rstar::Point> KNN<T>
where
    T: Point<Scalar = Float>,
{
    /// Create an algorithm to classify new features into one of the labels
    ///
    /// Every time this function is run a new algorithm is generated, rather than updating or
    /// adding points to the existing one.
    ///
    pub fn fit(&mut self, features: &[T], labels: &[usize]) {
        let values: Vec<Features<T>> = izip!(features, labels)
            .map(|(&feat, &class)| Features::new(feat, class))
            .collect();

        self.tree = Some(RTree::bulk_load(values));
    }

    pub fn predict(&self, features: &[T]) -> Result<Vec<usize>, Error> {
        if let Some(tree) = &self.tree {
            // Find the k-Nearest Neighbours
            Ok(features
                .iter()
                .map(|feat| {
                    let values: Vec<_> = tree
                        .nearest_neighbor_iter(feat)
                        .take(self.k)
                        .map(|x| x.label)
                        .collect();

                    let mut counts = vec![0_usize; *values.iter().max().unwrap() + 1];
                    for i in values {
                        counts[i] += 1;
                    }
                    counts
                        .iter()
                        .enumerate()
                        .max_by_key(|&(_, item)| item)
                        .unwrap()
                        .0
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

    #[test]
    fn simple_classification() -> Result<(), Error> {
        let mut knn = KNN::default();
        knn.fit(&vec![[0.; 2]; 10], &vec![0; 10]);
        assert_eq!(knn.predict(&vec![[0.; 2]; 5])?, [0; 5]);
        Ok(())
    }

    #[test]
    fn dual_classification() -> Result<(), Error> {
        let mut knn = KNN::default();
        let mut features = vec![[0.; 2]; 10];
        features.extend(&vec![[1.; 2]; 10]);
        let mut classes = vec![0; 10];
        classes.extend(&vec![1; 10]);
        knn.fit(&features, &classes);
        assert_eq!(knn.predict(&vec![[0.; 2]; 5])?, [0; 5]);
        assert_eq!(knn.predict(&vec![[1.; 2]; 5])?, [1; 5]);
        Ok(())
    }

    #[test]
    fn messy_classification() -> Result<(), Error> {
        let mut knn = KNN::default();
        let mut features = vec![[0.; 2]; 10];
        features.extend(&vec![[1.; 2]; 10]);
        features[0] = [1., 1.];
        features[10] = [0., 0.];
        let mut classes = vec![0; 10];
        classes.extend(&vec![1; 10]);
        knn.fit(&features, &classes);
        assert_eq!(knn.predict(&vec![[0.; 2]; 5])?, [0; 5]);
        assert_eq!(knn.predict(&vec![[1.; 2]; 5])?, [1; 5]);
        Ok(())
    }
}
