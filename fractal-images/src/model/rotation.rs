#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Rotation {
    By0,
    By90,
    By180,
    By270,
}

impl TryFrom<u8> for Rotation {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rotation::By0),
            1 => Ok(Rotation::By90),
            2 => Ok(Rotation::By180),
            3 => Ok(Rotation::By270),
            _ => Err("Invalid value for Rotation"),
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

    #[test]
    fn test_u8_to_rotation() {
        assert_eq!(Rotation::try_from(0), Ok(Rotation::By0));
        assert_eq!(Rotation::try_from(1), Ok(Rotation::By90));
        assert_eq!(Rotation::try_from(2), Ok(Rotation::By180));
        assert_eq!(Rotation::try_from(3), Ok(Rotation::By270));
        assert_eq!(Rotation::try_from(4), Err("Invalid value for Rotation"));
    }

    #[test]
    fn test_rotation_to_u8() {
        assert_eq!(u8::from(Rotation::By0), 0);
        assert_eq!(u8::from(Rotation::By90), 1);
        assert_eq!(u8::from(Rotation::By180), 2);
        assert_eq!(u8::from(Rotation::By270), 3);
    }
}

