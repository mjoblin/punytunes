use std::ops::Index;

/// Calculate a running average.

pub struct RunningAverage {
    pub values: Vec<f64>,
    limit: usize,
    position: usize,
}

impl RunningAverage {
    pub fn new(limit: usize) -> Self {
        RunningAverage {
            values: Vec::new(),
            limit,
            position: 0,
        }
    }

    pub fn add(&mut self, value: f64) {
        self.values.push(value);

        if self.values.len() > self.limit {
            self.values.remove(0);
        }
    }

    pub fn average(&self) -> f64 {
        let sum: f64 = self.values.iter().sum();

        sum / self.values.len() as f64
    }
}

impl Index<usize> for RunningAverage {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl Iterator for RunningAverage {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.values.len() {
            let value = self.values[self.position];
            self.position += 1;

            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_items = self.values.len() - self.position;
        (remaining_items, Some(remaining_items))
    }
}

impl ExactSizeIterator for RunningAverage {
    fn len(&self) -> usize {
        self.values.len()
    }
}

// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::average::RunningAverage;

    #[test]
    fn it_iterates() {
        let item_count = 5;
        let mut avg = RunningAverage::new(item_count);

        for i in 0..item_count {
            avg.add(i as f64);
        }

        let mut iteration_count = 0;

        for (index, value) in avg.enumerate() {
            assert_eq!(index as f64, value);
            iteration_count += 1;
        }

        assert_eq!(iteration_count, item_count);
    }

    #[test]
    fn it_maintains_limit() {
        let item_count = 2;

        let mut avg = RunningAverage::new(item_count);
        assert_eq!(avg.len(), 0);

        avg.add(1f64);
        assert_eq!(avg.len(), 1);
        avg.add(1f64);
        assert_eq!(avg.len(), 2);
        avg.add(1f64);
        assert_eq!(avg.len(), 2);
    }

    #[test]
    fn it_can_be_indexed() {
        let mut avg = RunningAverage::new(2);

        avg.add(1f64);
        avg.add(2f64);

        assert_eq!(avg[0], 1f64);
        assert_eq!(avg[1], 2f64);
    }

    #[test]
    fn it_removes_from_front_when_exceeding_limit() {
        let mut avg = RunningAverage::new(2);

        avg.add(1f64);
        avg.add(2f64);
        avg.add(3f64);

        assert_eq!(avg[0], 2f64);
        assert_eq!(avg[1], 3f64);
    }

    #[test]
    fn it_computes_average_with_fewer_items_than_limit() {
        let mut avg = RunningAverage::new(3);

        avg.add(2f64);
        avg.add(8f64);

        assert_eq!(avg.average(), 5f64);
    }

    #[test]
    fn it_computes_average_with_same_items_as_limit() {
        let mut avg = RunningAverage::new(3);

        avg.add(3f64);
        avg.add(4f64);
        avg.add(8f64);

        assert_eq!(avg.average(), 5f64);
    }

    #[test]
    fn it_computes_average_with_more_items_than_limit() {
        let mut avg = RunningAverage::new(3);

        avg.add(10f64);
        avg.add(3f64);
        avg.add(4f64);
        avg.add(8f64);

        assert_eq!(avg.average(), 5f64);
    }

    fn it_supports_ints() {
        let mut avg = RunningAverage::new(3);

        avg.add(10f64);
        avg.add(3f64);
        avg.add(4f64);
        avg.add(8f64);

        assert_eq!(avg.average(), 5f64);
    }
}
