use ndarray::{Array, Array2, Ix1};

pub struct MatrixOps;

impl MatrixOps {
    pub fn zeros(rows: usize, cols: usize) -> Array2<f32> {
        Array::zeros((rows, cols))
    }

    pub fn ones(rows: usize, cols: usize) -> Array2<f32> {
        Array::ones((rows, cols))
    }

    pub fn identity(size: usize) -> Array2<f32> {
        Array::eye(size)
    }

    pub fn add(a: &Array2<f32>, b: &Array2<f32>) -> Array2<f32> {
        a + b
    }

    pub fn multiply(a: &Array2<f32>, b: &Array2<f32>) -> Array2<f32> {
        a.dot(b)
    }

    pub fn multiply_scalar(a: &Array2<f32>, scalar: f32) -> Array2<f32> {
        a * scalar
    }

    pub fn transpose(a: &Array2<f32>) -> Array2<f32> {
        a.t().to_owned()
    }

    pub fn relu(a: &Array2<f32>) -> Array2<f32> {
        a.mapv(|x| x.max(0.0))
    }

    pub fn softmax(a: &Array2<f32>) -> Array2<f32> {
        let max = a.fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));
        let exp = a.mapv(|x| (x - max).exp());
        let sum: f32 = exp.iter().sum();
        exp / sum
    }

    pub fn linear(
        x: &Array2<f32>,
        weight: &Array2<f32>,
        bias: Option<&Array2<f32>>,
    ) -> Array2<f32> {
        let result = x.dot(weight);
        match bias {
            Some(b) => result + b,
            None => result,
        }
    }
}

pub type Array1<T> = Array<T, ndarray::Ix1>;
pub type Array3<T> = Array<T, ndarray::Ix3>;
