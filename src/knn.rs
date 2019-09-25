//
// knn.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

//! Implement a K-NN classification algorithm

use failure::{err_msg, Error};
use itertools::izip;
use rstar::{Point, PointDistance, RTree, RTreeObject, AABB};

type Float = f32;

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
        Features {
            label: label,
            features,
        }
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
    pub fn fit(&mut self, X: &[T], y: &[usize]) {
        let values: Vec<Features<T>> = izip!(X, y)
            .map(|(&feat, &class)| Features::new(feat, class))
            .collect();

        self.tree = Some(RTree::bulk_load(values));
    }

    pub fn predict(&self, X: &[T]) -> Result<Vec<usize>, Error> {
        if let Some(tree) = &self.tree {
            // Find the k-Nearest Neighbours
            Ok(X.iter()
                .map(|feat| {
                    let values: Vec<_> = tree
                        .nearest_neighbor_iter(feat)
                        .take(self.k)
                        .map(|x| x.label)
                        .collect();

                    let mut counts = vec![0_usize; *values.iter().max().unwrap()];
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
    fn it_works() {}
}
