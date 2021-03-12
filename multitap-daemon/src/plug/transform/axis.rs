use super::*;

type CreateResult = Result<(AxisId, AxisTransform), Error>;

pub enum AxisTransform {
    IntoAxis(IntoAxis),
    IntoKeys(IntoKeys),
}

impl AxisTransform {
    pub fn create(context: &TransformCreateContext, mapper: &Mapper) -> CreateResult {
        match mapper {
            Mapper::Axis(input, output) => IntoAxis::create(context, false, input, output),
            Mapper::AxisReversed(input, output) => IntoAxis::create(context, true, input, output),
            Mapper::AxisToKeys(input, output_min, output_max, threshold) => IntoKeys::create(context, input, output_min, output_max, *threshold),
            Mapper::HatToKeys(input, output_min, output_max) => IntoKeys::create_hat(context, input, output_min, output_max),
            _ => Err(Error::InvalidTransformCreateCalled),
        }
    }
}

impl Transform<AxisEvent> for AxisTransform {
    fn apply(&mut self, event: &AxisEvent) -> Result<Vec<InputEvent>> {
        use AxisTransform::*;
        match self {
            IntoAxis(into_axis) => into_axis.apply(event),
            IntoKeys(into_keys) => into_keys.apply(event),
        }
    }
}

pub struct IntoAxis {
    id: AxisId,
    reverse: bool,
    from_min: isize,
    from_range: isize,
    to_min: isize,
    to_range: isize,
}

pub struct IntoKeys(Vec<IntoKey>);
struct IntoKey {
    lower: isize, 
    upper: isize, 
    id: KeyId, 
    state: KeyState
}

impl IntoAxis {
    pub fn create(context: &TransformCreateContext, reverse: bool, input: &str, output: &str) -> CreateResult {
        let input_axis = context.find_input_axis(input)?;
        let input_axisinfo = context.find_input_axisinfo(input_axis)?;
        let output_axis = context.find_output_axis(output)?;

        let from_min = input_axisinfo.min as isize;
        let from_max = input_axisinfo.max as isize;
        let to_min = output_axis.min as isize;
        let to_max = output_axis.max as isize;
        let from_range = from_max - from_min;
        let to_range = to_max - to_min;

        let into_axis = Self { id: output_axis.id, reverse, from_min, from_range, to_min, to_range };
        Ok((input_axis, AxisTransform::IntoAxis(into_axis)))
    }

    fn apply(&mut self, event: &AxisEvent) -> Result<Vec<InputEvent>> {
        let id = self.id;
        let state = (event.state.0 - self.from_min) * self.to_range / self.from_range + self.to_min;
        let state = if self.reverse { -state } else { state };
        let state = AxisState(state);
        single(AxisEvent { timestamp: event.timestamp, id, state })
    }
}

impl IntoKeys {
    pub fn create(context: &TransformCreateContext, input: &str, output_min: &str, output_max: &str, threshold: isize) -> CreateResult {
        let input_axis = context.find_input_axis(input)?;
        let input_axisinfo = context.find_input_axisinfo(input_axis)?;
        let output_min_key = context.find_output_key(output_min)?;
        let output_max_key = context.find_output_key(output_max)?;

        let input_min = input_axisinfo.min as isize;
        let input_max = input_axisinfo.max as isize;
        let input_halfrange = (input_max - input_min) / 2;
        let input_midpoint = (input_max + input_min) / 2;
        let input_min_threshold = input_midpoint - input_halfrange * threshold / 100;
        let input_max_threshold = input_midpoint + input_halfrange * threshold / 100;

        let into_keys = Self::new(&[
            (input_min, input_min_threshold, output_min_key),
            (input_max_threshold, input_max, output_max_key),
        ]);
        Ok((input_axis, AxisTransform::IntoKeys(into_keys)))
    }

    pub fn create_hat(context: &TransformCreateContext, input: &str, output_min: &str, output_max: &str) -> CreateResult {
        let input_axis = context.find_input_axis(input)?;
        let input_axisinfo = context.find_input_axisinfo(input_axis)?;
        let output_min_key = context.find_output_key(output_min)?;
        let output_max_key = context.find_output_key(output_max)?;
        let into_keys = Self::new(&[
            ((input_axisinfo.min as isize), -1, output_min_key),
            (1, (input_axisinfo.max as isize), output_max_key),
        ]);
        Ok((input_axis, AxisTransform::IntoKeys(into_keys)))
    }

    fn new(keys: &[(isize, isize, KeyId)]) -> Self {
        let keys = keys.iter().map(|(lower, upper, id)| IntoKey {
            lower: *lower,
            upper: *upper,
            id: *id, 
            state: KeyState::Released
        }).collect();
        Self(keys)
    }

    fn apply(&mut self, event: &AxisEvent) -> Result<Vec<InputEvent>> {
        let timestamp = event.timestamp;
        let axis_state = event.state.0 as isize;
        let mut events = vec![];
        for key in self.0.iter_mut() {
            let id = key.id;
            let state = if axis_state >= key.lower && axis_state <= key.upper { KeyState::Pressed } else { KeyState::Released };
            if key.state != state {
                key.state = state;
                events.push(KeyEvent { timestamp, id, state }.into());
            }
        }

        Ok(events)
    }
}
