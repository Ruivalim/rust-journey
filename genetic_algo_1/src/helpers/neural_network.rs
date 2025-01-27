use ndarray::{Array1, Array2};
use rand::Rng;

#[derive(Clone, Debug)]
pub struct NeuralNetwork {
    pub weights_input_hidden: Array2<f32>,
    pub weights_hidden_output: Array2<f32>,
    pub biases_hidden: Array1<f32>,
    pub biases_output: Array1<f32>,
}

impl NeuralNetwork {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            weights_input_hidden: Array2::from_shape_fn((input_size, hidden_size), |_| {
                rng.gen_range(-1.0..1.0)
            }),
            weights_hidden_output: Array2::from_shape_fn((hidden_size, output_size), |_| {
                rng.gen_range(-1.0..1.0)
            }),
            biases_hidden: Array1::from_shape_fn(hidden_size, |_| rng.gen_range(-1.0..1.0)),
            biases_output: Array1::from_shape_fn(output_size, |_| rng.gen_range(-1.0..1.0)),
        }
    }

    pub fn feedforward(&self, inputs: &Array1<f32>) -> Array1<f32> {
        let hidden_inputs = inputs.dot(&self.weights_input_hidden) + &self.biases_hidden;
        let hidden_outputs = hidden_inputs.mapv(Self::sigmoid);

        let final_inputs = hidden_outputs.dot(&self.weights_hidden_output) + &self.biases_output;
        final_inputs.mapv(Self::sigmoid)
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }
}
