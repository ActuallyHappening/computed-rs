mod examples {
    use computed::Computed;
    use derive_new::new;
    use std::sync::OnceLock;

    // provided

    #[derive(Computed, new)]
    pub struct Example {
        #[computed(get, set, invalidates(sum))]
        items: Vec<f32>,

        #[computed(computed(Example::_compute_sum))]
        #[new(default)]
        sum: OnceLock<f32>,
    }

    impl Example {
        fn _compute_sum(field: &Vec<f32>) -> f32 {
            field.iter().sum()
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

    // // computed storage
    // impl Example {
    // 	pub fn compute_sum(&self) -> &f32 {
    // 		self.sum.get_or_init(|| Example::_compute_sum(&self.items))
    // 	}
    // }
}

use computed::Computed;
use examples::*;

fn main() {
    // let example = Example::new(vec![1., 2., 3.]);
    // let _ = example.get_items()[0];
}
