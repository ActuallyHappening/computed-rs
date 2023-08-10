mod examples {
    use computed::Computed;
    use derive_new::new;
    use std::sync::OnceLock;

    // provided

    #[derive(Computed, new)]
    pub struct Example {
        #[computed(get, set, invalidates(sum))]
        items: Vec<f32>,

        #[computed(get, set, invalidates(sum))]
        discount: f32,

        #[computed(computed(Example::_compute_sum))]
        #[new(default)]
        sum: OnceLock<f32>,
    }

    impl Example {
        fn _compute_sum(items: &Vec<f32>, discount: &f32) -> f32 {
            items.iter().sum::<f32>() * (1. - discount)
        }
    }

    // // generated

    // // getters / setters
    // impl Example {
    // 	pub fn get_items(&self) -> &Vec<f32> {
    // 		&self.items
    // 	}

    // 	pub fn set_items(&mut self, items: Vec<f32>) {
    // 		self.items = items;

    // 		self.sum = OnceLock::new();
    // 	}
    // }

    // computed storage
    impl Example {
        pub fn compute_sum(&self) -> &f32 {
            self.sum
                .get_or_init(|| Example::_compute_sum(&self.items, &self.discount))
        }
    }

    #[test]
    fn test_internals() {
        let example = Example::new(vec![0., 10., 0.], 0.);
        example.compute_sum();
        assert_eq!(example.sum.get(), Some(&10.));
    }
}

use computed::Computed;
use examples::*;

fn main() {
    let mut example = Example::new(vec![1., 2., 3.], 0.);
    assert_eq!(example.get_items()[0], 1.);

    example.set_items(vec![4., 5., 6.]);
    assert_eq!(example.get_items()[0], 4.);

    let sum = example.compute_sum();
    assert_eq!(*sum, 15.);
}
