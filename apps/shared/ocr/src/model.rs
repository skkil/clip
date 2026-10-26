use std::sync::{Arc, Mutex};

use tract_onnx::prelude::*;

use crate::{Error, Result};

const CACHED_SHAPES: usize = 32;

type Plan = Arc<TypedRunnableModel>;

#[derive(Debug)]
pub(crate) struct Model {
    graph: InferenceModel,
    plans: Mutex<Vec<([usize; 4], Plan)>>,
}

impl Model {
    pub(crate) fn load(onnx_bytes: &[u8]) -> Result<Self> {
        let graph = tract_onnx::onnx()
            .with_ignore_output_shapes(true)
            .with_ignore_value_info(true)
            .model_for_read(&mut &onnx_bytes[..])?;
        Ok(Self {
            graph,
            plans: Mutex::new(Vec::new()),
        })
    }

    pub(crate) fn run(&self, shape: [usize; 4], input: &[f32]) -> Result<Tensor> {
        let plan = self.plan(shape)?;
        let input = Tensor::from_shape(&shape, input)?;
        let mut output = plan.run(tvec!(input.into()))?;
        Ok(output.remove(0).into_tensor())
    }

    fn plan(&self, shape: [usize; 4]) -> Result<Plan> {
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| Error::Model("plan cache poisoned".into()))?;
        if let Some(i) = plans.iter().position(|(s, _)| *s == shape) {
            let entry = plans.remove(i);
            let plan = entry.1.clone();
            plans.push(entry);
            return Ok(plan);
        }
        let plan = self
            .graph
            .clone()
            .with_input_fact(0, f32::fact(shape).into())?
            .into_optimized()?
            .into_runnable()?;
        if plans.len() == CACHED_SHAPES {
            plans.remove(0);
        }
        plans.push((shape, plan.clone()));
        Ok(plan)
    }
}

pub(crate) fn as_slice(tensor: &Tensor) -> Result<&[f32]> {
    tensor
        .to_plain_array_view::<f32>()?
        .to_slice()
        .ok_or_else(|| Error::Model("non-contiguous output".into()))
}
