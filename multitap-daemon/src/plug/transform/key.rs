use super::*;

type CreateResult = Result<(KeyId, KeyTransform), Error>;

pub enum KeyTransform {
    IntoKey(IntoKey),
    IntoReversedKey(IntoReversedKey),
    IntoAxis(IntoAxis),
}

pub struct IntoKey(KeyId);
pub struct IntoReversedKey(KeyId);
pub struct IntoAxis { pub id: AxisId, pub pressed: isize, pub released: isize }

impl KeyTransform {
    pub fn create(context: &TransformCreateContext, mapper: &Mapper) -> CreateResult {
        match mapper {
            Mapper::Key(input, output) => IntoKey::create(context, input, output),
            Mapper::KeyReversed(input, output) => IntoReversedKey::create(context, input, output),
            Mapper::KeyToAxis(input, output) => IntoAxis::create(context, input, output),
            _ => Err(Error::InvalidTransformCreateCalled),
        }
    }
}

impl Transform<KeyEvent> for KeyTransform {
    fn apply(&mut self, event: &KeyEvent) -> Result<Vec<InputEvent>> {
        use KeyTransform::*;
        match self {
            IntoKey(transform) => transform.apply(event),
            IntoReversedKey(transform) => transform.apply(event),
            IntoAxis(transform) => transform.apply(event),
        }
    }
}

impl IntoKey {
    pub fn create(context: &TransformCreateContext, input: &str, output: &str) -> CreateResult {
        let input_key = context.find_input_key(input)?;
        let output_key = context.find_output_key(output)?;
        Ok((input_key, KeyTransform::IntoKey(Self(output_key))))
    }

    fn apply(&self, event: &KeyEvent) -> Result<Vec<InputEvent>> {
        single(KeyEvent { timestamp: event.timestamp, id: self.0, state: event.state })
    }
}

impl IntoReversedKey {
    pub fn create(context: &TransformCreateContext, input: &str, output: &str) -> CreateResult {
        let input_key = context.find_input_key(input)?;
        let output_key = context.find_output_key(output)?;
        Ok((input_key, KeyTransform::IntoReversedKey(Self(output_key))))
    }

    fn apply(&self, event: &KeyEvent) -> Result<Vec<InputEvent>> {
        let state = match event.state {
            KeyState::Pressed  => KeyState::Released,
            KeyState::Released => KeyState::Pressed,
            KeyState::AutoRepeat => return Err(Error::UnsupportedAutoRepeatForReverseKey(self.0)),
        };
        single(KeyEvent { timestamp: event.timestamp, id: self.0, state })
    }
}

impl IntoAxis {
    pub fn create(context: &TransformCreateContext, input: &str, output: &str) -> CreateResult {
        let input_key = context.find_input_key(input)?;
        let output_axis = context.find_output_axis(output)?;
        Ok((input_key, KeyTransform::IntoAxis(Self {
            id:       output_axis.id,
            released: output_axis.min as isize,
            pressed:  output_axis.max as isize,
        })))
    }

    fn apply(&self, event: &KeyEvent) -> Result<Vec<InputEvent>> {
        let state = match event.state {
            KeyState::Pressed | KeyState::AutoRepeat => AxisState(self.pressed),
            KeyState::Released => AxisState(self.released),
        };
        single(AxisEvent { timestamp: event.timestamp, id: self.id, state })
    }
}