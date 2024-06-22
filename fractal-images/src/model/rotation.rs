use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Rotation {
    By0,
    By90,
    By180,
    By270,
}

#[derive(Error, Debug, Eq, PartialEq, )]
#[error("Unknown rotation code: {}", {.code})]
pub struct RotationInvalidError {
    code: u8,
}

impl TryFrom<u8> for Rotation {
    type Error = RotationInvalidError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rotation::By0),
            1 => Ok(Rotation::By90),
            2 => Ok(Rotation::By180),
            3 => Ok(Rotation::By270),
            code => Err(RotationInvalidError { code }),
        }
    }
}


impl From<Rotation> for u8 {
    fn from(value: Rotation) -> Self {
        match value {
            Rotation::By0 => 0,
            Rotation::By90 => 1,
            Rotation::By180 => 2,
            Rotation::By270 => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use fluid::prelude::*;

    #[theory]
    #[case(0, Rotation::By0)]
    #[case(1, Rotation::By90)]
    #[case(2, Rotation::By180)]
    #[case(3, Rotation::By270)]
    fn u8_converts_to_rotation(val: u8, rotation: Rotation) {
        let result = Rotation::try_from(val);
        result.as_ref().should().be_ok()
            .because("it is a valid rotation code");

        let result = result.unwrap();
        result.should().be_equal_to(rotation)
            .because("the rotation is mapped to that code");
    }

    #[theory]
    #[case(Rotation::By0, 0)]
    #[case(Rotation::By90, 1)]
    #[case(Rotation::By180, 2)]
    #[case(Rotation::By270, 3)]
    fn rotation_converts_to_u8(rotation: Rotation, val: u8) {
        u8::from(rotation).should().be_equal_to(val);
    }
}

