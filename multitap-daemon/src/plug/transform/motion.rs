use super::*;

type CreateResult = Result<(MotionId, MotionTransform), Error>;

pub enum MotionTransform {
    IntoMotion(IntoMotion),
}

pub struct IntoMotion(MotionId);

impl MotionTransform {
    pub fn create(context: &TransformCreateContext, mapper: &Mapper) -> CreateResult {
        match mapper {
            Mapper::Motion(input, output) => IntoMotion::create(context, input, output),
            _ => Err(Error::InvalidTransformCreateCalled),
        }
    }
}

impl Transform<MotionEvent> for MotionTransform {
    fn apply(&mut self, event: &MotionEvent) -> Result<Vec<InputEvent>> {
        use MotionTransform::*;
        match self {
            IntoMotion(transform) => transform.apply(event),
        }
    }
}

impl IntoMotion {
    pub fn create(context: &TransformCreateContext, input: &str, output: &str) -> CreateResult {
        let input_motion = context.find_input_motion(input)?;
        let output_motion = context.find_output_motion(output)?;
        Ok((input_motion, MotionTransform::IntoMotion(Self(output_motion))))
    }

    fn apply(&self, event: &MotionEvent) -> Result<Vec<InputEvent>> {
        single(MotionEvent { timestamp: event.timestamp, id: self.0, state: event.state })
    }
}
